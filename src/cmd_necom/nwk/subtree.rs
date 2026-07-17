use anyhow::Context;
use clap::{value_parser, Arg, ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
use std::io::Write;

/// Build the clap subcommand for subtree.
pub fn make_subcommand() -> Command {
    Command::new("subtree")
        .about("Extracts a subtree")
        .after_help(include_str!("../../../docs/help/nwk/subtree.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::node_arg())
        .arg(crate::cmd_necom::args::name_list_arg())
        .arg(crate::cmd_necom::args::regex_arg())
        .arg(crate::cmd_necom::args::descendants_arg())
        .arg(crate::cmd_necom::args::monophyly_arg(
            "Only print the subtree when it's a clade",
        ))
        .arg(
            Arg::new("condense")
                .long("condense")
                .short('C')
                .num_args(1)
                .help("Condense the subtree into a single node with this name"),
        )
        .arg(
            Arg::new("context")
                .long("context")
                .short('c')
                .num_args(1)
                .value_parser(value_parser!(usize))
                .default_value("0")
                .help("Extend the subtree by N levels above the LCA"),
        )
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the subtree command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    let is_monophyly = args.get_flag("monophyly");
    let condense_name = args.get_one::<String>("condense");
    let is_condense = condense_name.is_some();

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let mut trees = Tree::from_file(infile)?;

    if trees.is_empty() {
        return Ok(());
    }

    for tree in &mut trees {
        // IDs matching names
        let ids = super::common::match_names(tree, args)?;

        if ids.is_empty() {
            continue;
        }

        // Find LCA
        let ids_vec: Vec<usize> = ids.iter().cloned().collect();
        let mut sub_root_id = tree.get_lca(&ids_vec)?;

        // Monophyly check: selected nodes must form a clade.
        if is_monophyly && !tree.is_clade(&ids_vec) {
            continue;
        }

        // Apply context
        let context_levels = *args
            .get_one::<usize>("context")
            .ok_or_else(|| anyhow::anyhow!("missing required argument: context"))?;
        for _ in 0..context_levels {
            if let Some(node) = tree.get_node(sub_root_id) {
                if let Some(parent) = node.parent {
                    sub_root_id = parent;
                } else {
                    break;
                }
            }
        }

        if is_condense {
            // condense_name is Some here (is_condense == condense_name.is_some()).
            let name = condense_name.map_or("", |s| s.as_str());
            if name.is_empty() {
                anyhow::bail!("--condense requires a non-empty name argument");
            }

            // Count named nodes in the selected set, matching the documented semantics.
            let member_count = ids
                .iter()
                .filter(|&&id| {
                    tree.get_node(id).map(|n| n.name.is_some()).unwrap_or(false)
                })
                .count();
            tree.condense_subtree(sub_root_id, name, member_count)?;

            let out_string = tree.to_newick();
            writer.write_fmt(format_args!("{}\n", out_string))?;
        } else {
            // Extract subtree
            let out_string = tree.to_newick_subtree(sub_root_id);
            writer.write_fmt(format_args!("{}\n", out_string))?;
        }
    }

    writer.flush()?;
    Ok(())
}
