use anyhow::Context;
use clap::{Arg, ArgAction, ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

/// Build the clap subcommand for reroot.
pub fn make_subcommand() -> Command {
    Command::new("reroot")
        .about("Reroots a tree at a specified node or the longest branch")
        .after_help(include_str!("../../../docs/help/nwk/reroot.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::node_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
        .arg(
            Arg::new("support_as_labels")
                .long("support-as-labels")
                .action(ArgAction::SetTrue)
                .help("Treat internal node labels as support values and shift them when rerooting"),
        )
        .arg(
            Arg::new("deroot")
                .long("deroot")
                .short('d')
                .action(ArgAction::SetTrue)
                .help("Deroot the tree (create a multifurcating root) (see Notes)"),
        )
        .arg(
            Arg::new("lax")
                .long("lax")
                .short('l')
                .action(ArgAction::SetTrue)
                .help("Lax mode: Use the complement if the specified nodes form the root (see Notes)"),
        )
}

/// Execute the reroot command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;
    let process_support = args.get_flag("support_as_labels");
    let deroot = args.get_flag("deroot");
    let lax = args.get_flag("lax");

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let trees = Tree::from_file(infile)?;

    if trees.len() > 1 {
        log::warn!(
            "file contains {} trees, only the first will be processed",
            trees.len()
        );
    }

    let mut tree = trees
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("no trees found in {}", infile))?;

    if deroot {
        tree.deroot()
            .map_err(|e| anyhow::anyhow!("deroot failed: {}", e))?;
    } else {
        // ids with names
        let id_of: BTreeMap<_, _> = tree.get_name_id();
        let duplicates = necom::libs::phylo::tree::stat::duplicate_names(&tree);

        // All IDs matched
        let mut ids = BTreeSet::new();
        let user_specified = args.get_many::<String>("node").is_some();
        if let Some(nodes) = args.get_many::<String>("node") {
            for name in nodes {
                if let Some(&id) = id_of.get(name) {
                    super::common::warn_duplicate_name(&duplicates, name);
                    ids.insert(id);
                } else {
                    log::warn!("node name not found in tree: {}", name);
                }
            }
        }

        if !ids.is_empty() {
            necom::libs::phylo::tree::ops::reroot_at_lca(
                &mut tree,
                &ids,
                lax,
                process_support,
            )?;
        } else if user_specified {
            anyhow::bail!("none of the specified --node names were found in the tree");
        } else {
            necom::libs::phylo::tree::ops::reroot_at_longest_branch(
                &mut tree,
                process_support,
            )?;
        }
    }

    let out_string = tree.to_newick();
    writer.write_fmt(format_args!("{}\n", out_string))?;

    writer.flush()?;
    Ok(())
}
