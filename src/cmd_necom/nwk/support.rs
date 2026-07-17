use anyhow::Context;
use clap::{Arg, ArgAction, ArgMatches, Command};
use necom::libs::phylo::tree::{self, support, Tree};
use std::collections::BTreeSet;
use std::io::Write;

/// Collect the set of named leaf labels in `tree`.
fn leaf_name_set(tree: &Tree) -> BTreeSet<String> {
    tree.get_leaf_names().into_iter().flatten().collect()
}

/// Build the clap subcommand for support.
pub fn make_subcommand() -> Command {
    Command::new("support")
        .about("Assigns bootstrap support values to internal nodes")
        .after_help(
            r###"
Attributes bootstrap support values to a target tree based on a set of replicate trees.

Notes:
* The first argument is the target tree (to which support values are attributed).
* The second argument is the replicate trees (e.g., from bootstrap).
* Support values are written as internal node labels.
* Assumes that all trees have the same set of leaves.

Examples:
1. Attribute support values:
   necom nwk support target.nwk replicates.nwk

2. Output support as percentages:
   necom nwk support target.nwk replicates.nwk --percent
"###,
        )
        .arg(crate::cmd_necom::args::target_tree_arg("Target tree file"))
        .arg(
            Arg::new("replicates")
                .required(true)
                .num_args(1)
                .index(2)
                .help("Replicate trees file"),
        )
        .arg(
            Arg::new("percent")
                .short('p')
                .long("percent")
                .action(ArgAction::SetTrue)
                .help("Print values as percentages"),
        )
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Execute the support command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let target_file = args
        .get_one::<String>("target")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: target"))?;
    let replicates_file = args
        .get_one::<String>("replicates")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: replicates"))?;
    let percent = args.get_flag("percent");

    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer =
        necom::writer(outfile).with_context(|| format!("Failed to open writer for {}", outfile))?;

    // 1. Read Replicate Trees
    // We read replicates first to build the leaf map and counts, similar to nw_support logic
    let replicates = tree::io::from_file(replicates_file)?;
    if replicates.is_empty() {
        anyhow::bail!("No replicate trees found");
    }
    let total_reps = replicates.len();

    // 2. Read Target Trees
    let mut targets = tree::io::from_file(target_file)?;
    if targets.is_empty() {
        anyhow::bail!("No target trees found");
    }

    // 3. Build Leaf Map (from first replicate)
    let leaf_map = support::build_leaf_map(&replicates[0])
        .map_err(|e| anyhow::anyhow!("build_leaf_map failed: {}", e))?;

    // 3.5 Validate that every target tree shares the same leaf set as the replicates.
    let replicate_leaves: BTreeSet<String> = replicates.iter().flat_map(leaf_name_set).collect();
    for (i, target) in targets.iter().enumerate() {
        let target_leaves = leaf_name_set(target);
        if target_leaves != replicate_leaves {
            let only_target: Vec<_> = target_leaves.difference(&replicate_leaves).collect();
            let only_replicates: Vec<_> = replicate_leaves.difference(&target_leaves).collect();
            anyhow::bail!(
                "target tree {} leaf set differs from replicate trees: only in target {:?}, only in replicates {:?}",
                i + 1,
                only_target,
                only_replicates
            );
        }
    }

    // 4. Count Clades in Replicates
    let counts = support::count_clades(&replicates, &leaf_map)
        .map_err(|e| anyhow::anyhow!("count_clades failed: {}", e))?;

    // 5. Annotate Target Trees
    for target in &mut targets {
        support::annotate_support(target, &leaf_map, &counts, total_reps, percent)
            .map_err(|e| anyhow::anyhow!("annotate_support failed: {}", e))?;
        writer.write_fmt(format_args!("{}\n", target.to_newick()))?;
    }

    writer.flush()?;
    Ok(())
}
