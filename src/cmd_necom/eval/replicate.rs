use anyhow::Context;
use clap::{Arg, ArgAction, ArgMatches, Command};
use necom::libs::phylo::tree::{support, Tree};
use std::collections::BTreeSet;
use std::io::{Read, Write};

/// Collect the set of named leaf labels in `tree`.
fn leaf_name_set(tree: &Tree) -> BTreeSet<String> {
    tree.get_leaf_names().into_iter().flatten().collect()
}

/// Load trees from `infile`, bailing with a clear message when the input is
/// empty or contains no parseable trees.
fn load_trees(infile: &str) -> anyhow::Result<Vec<Tree>> {
    let mut reader =
        necom::reader(infile).with_context(|| format!("failed to open {}", infile))?;
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .with_context(|| format!("failed to read {}", infile))?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    Tree::from_newick_multi(&content)
        .with_context(|| format!("failed to parse '{}'", infile))
}

/// Build the clap subcommand for replicate.
pub fn make_subcommand() -> Command {
    Command::new("replicate")
        .about("Assigns support values to internal nodes from replicate trees")
        .after_help(include_str!("../../../docs/help/eval/replicate.md"))
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
        .arg(
            Arg::new("override_root")
                .short('r')
                .long("override-root")
                .action(ArgAction::SetTrue)
                .help("Override the root node label with its support value"),
        )
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Execute the replicate command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let target_file = args
        .get_one::<String>("target")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: target"))?;
    let replicates_file = args
        .get_one::<String>("replicates")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: replicates"))?;
    let percent = args.get_flag("percent");
    let override_root = args.get_flag("override_root");

    // We read replicates first to build the leaf map and counts, similar to nw_support logic
    let replicates = load_trees(replicates_file)?;
    if replicates.is_empty() {
        anyhow::bail!("No replicate trees found");
    }
    let total_reps = replicates.len();

    // 2. Read Target Trees
    let mut targets = load_trees(target_file)?;
    if targets.is_empty() {
        anyhow::bail!("No target trees found");
    }

    // 3. Build Leaf Map (from first replicate)
    let leaf_map = support::build_leaf_map(&replicates[0])
        .with_context(|| "build_leaf_map failed")?;

    // 3.5 Validate that all replicate trees share the same leaf set.
    let first_replicate_leaves = leaf_name_set(&replicates[0]);
    for (i, rep) in replicates.iter().enumerate().skip(1) {
        let rep_leaves = leaf_name_set(rep);
        if rep_leaves != first_replicate_leaves {
            let only_rep: Vec<_> =
                rep_leaves.difference(&first_replicate_leaves).collect();
            let only_first: Vec<_> =
                first_replicate_leaves.difference(&rep_leaves).collect();
            anyhow::bail!(
                "replicate tree {} leaf set differs from first replicate: only in replicate {:?}, only in first {:?}",
                i + 1,
                only_rep,
                only_first
            );
        }
    }

    // 3.6 Validate that every target tree shares the same leaf set as the replicates.
    let replicate_leaves = first_replicate_leaves;
    for (i, target) in targets.iter().enumerate() {
        let target_leaves = leaf_name_set(target);
        if target_leaves != replicate_leaves {
            let only_target: Vec<_> =
                target_leaves.difference(&replicate_leaves).collect();
            let only_replicates: Vec<_> =
                replicate_leaves.difference(&target_leaves).collect();
            anyhow::bail!(
                "target tree {} leaf set differs from replicate trees: only in target {:?}, only in replicates {:?}",
                i + 1,
                only_target,
                only_replicates
            );
        }
    }

    // Open the output writer only after input validation so that invalid
    // invocations do not truncate an existing output file.
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    // 4. Count Clades in Replicates
    let counts = support::count_clades(&replicates, &leaf_map)
        .with_context(|| "count_clades failed")?;

    // 5. Annotate Target Trees
    for target in &mut targets {
        support::annotate_support(
            target,
            &leaf_map,
            &counts,
            total_reps,
            percent,
            override_root,
        )
        .with_context(|| "annotate_support failed")?;
        writeln!(writer, "{}", target.to_newick())?;
    }

    writer.flush()?;
    Ok(())
}
