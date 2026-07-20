use anyhow::Context;
use clap::{ArgMatches, Command};
use necom::libs::phylo::tree::algo;
use necom::libs::phylo::tree::Tree;
use std::io::Write;

/// Build the clap subcommand for prune.
pub fn make_subcommand() -> Command {
    Command::new("prune")
        .about("Removes nodes from a Newick file")
        .after_help(include_str!("../../../docs/help/nwk/prune.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::node_arg())
        .arg(crate::cmd_necom::args::name_list_arg())
        .arg(crate::cmd_necom::args::regex_arg())
        .arg(crate::cmd_necom::args::descendants_arg())
        .arg(crate::cmd_necom::args::invert_arg_with_help(
            "Invert pruning: keep specified nodes, their ancestors and descendants",
        ))
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the prune command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let trees = Tree::from_file(infile)?;

    for mut tree in trees {
        let target_ids = super::common::match_names(&tree, args, false)?;

        if !args.try_contains_id("node").unwrap_or(false)
            && !args.try_contains_id("name_list").unwrap_or(false)
            && !args.try_contains_id("regex").unwrap_or(false)
        {
            // No selector given: pruning is destructive, so keep the tree
            // unchanged rather than defaulting to all named nodes.
            log::warn!("no --node/--name-list/--regex provided; leaving tree unchanged");
            writer.write_fmt(format_args!("{}\n", tree.to_newick()))?;
            continue;
        }

        let to_remove: Vec<_> = if args.get_flag("invert") {
            if target_ids.is_empty() {
                log::warn!(
                    "--invert set but no target nodes matched; keeping tree unchanged"
                );
                Vec::new()
            } else {
                let keep = algo::compute_keep_set(&tree, target_ids.iter().copied());
                match tree.get_root() {
                    Some(root) => {
                        let all_ids = tree.levelorder(root);
                        all_ids
                            .into_iter()
                            .filter(|id| !keep.contains(id))
                            .collect()
                    }
                    None => Vec::new(),
                }
            }
        } else {
            target_ids.into_iter().collect()
        };

        algo::prune_nodes(&mut tree, to_remove)?;

        let out_string = tree.to_newick();
        writer.write_fmt(format_args!("{}\n", out_string))?;
    }

    writer.flush()?;
    Ok(())
}
