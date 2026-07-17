use anyhow::Context;
use clap::{Arg, ArgMatches, Command};
use necom::libs::phylo::node::NodeId;
use necom::libs::phylo::tree::Tree;
use std::io::Write;

/// Build the clap subcommand for comment.
pub fn make_subcommand() -> Command {
    Command::new("comment")
        .about("Adds comments to node(s) in a Newick file")
        .after_help(include_str!("../../../docs/help/nwk/comment.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::node_arg())
        .arg(crate::cmd_necom::args::lca_arg())
        .arg(
            Arg::new("string")
                .long("string")
                .num_args(1)
                .help("Free-form string stored as a separate property"),
        )
        .arg(crate::cmd_necom::args::color_arg(None, "Color of names"))
        .arg(
            Arg::new("label")
                .long("label")
                .num_args(1)
                .help("Add this label to the south west of the node"),
        )
        .arg(
            Arg::new("comment_text")
                .long("comment-text")
                .num_args(1)
                .help("Comment text after names"),
        )
        .arg(
            Arg::new("dot")
                .long("dot")
                .num_args(0..=1)
                .default_missing_value("black")
                .help("Place a dot in the node; value as color"),
        )
        .arg(
            Arg::new("bar")
                .long("bar")
                .num_args(0..=1)
                .default_missing_value("black")
                .help("Place a bar in the middle of the parent edge; value as color"),
        )
        .arg(
            Arg::new("rec")
                .long("rec")
                .num_args(0..=1)
                .default_missing_value("LemonChiffon")
                .help(
                    "Place a rectangle in the background of the subtree; value as color",
                ),
        )
        .arg(
            Arg::new("tri")
                .long("tri")
                .num_args(0..=1)
                .default_missing_value("white")
                .help("Place a triangle at the end of the branch; value as color"),
        )
        .arg(
            Arg::new("remove")
                .long("remove")
                .num_args(1)
                .help("Scan all nodes and remove parts of comments matching the regex"),
        )
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the comment command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    let opt_string = args.get_one::<String>("string");

    let opt_label = args.get_one::<String>("label");
    let opt_color = args.get_one::<String>("color");
    let opt_comment = args.get_one::<String>("comment_text");

    let opt_dot = args.get_one::<String>("dot");
    let opt_bar = args.get_one::<String>("bar");
    let opt_rec = args.get_one::<String>("rec");
    let opt_tri = args.get_one::<String>("tri");

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let mut trees = Tree::from_file(infile)?;

    for tree in &mut trees {
        let ids: Vec<NodeId> =
            super::common::match_nodes_and_lca(tree, args, "node", "lca")?
                .into_iter()
                .collect();

        for id in &ids {
            if let Some(node) = tree.get_node_mut(*id) {
                if let Some(x) = opt_string {
                    node.add_property("string", x);
                }

                if let Some(x) = opt_label {
                    node.add_property("label", x);
                }
                if let Some(x) = opt_color {
                    node.add_property("color", x);
                }
                if let Some(x) = opt_comment {
                    node.add_property("comment", x);
                }

                if let Some(x) = opt_dot {
                    node.add_property("dot", x);
                }
                if let Some(x) = opt_bar {
                    node.add_property("bar", x);
                }
                if let Some(x) = opt_rec {
                    node.add_property("rec", x);
                }
                if let Some(x) = opt_tri {
                    node.add_property("tri", x);
                }
            }
        }

        // Remove parts of comments
        if args.contains_id("remove") {
            let pattern = args
                .get_one::<String>("remove")
                .ok_or_else(|| anyhow::anyhow!("missing required argument: remove"))?;
            necom::libs::phylo::tree::ops::remove_properties_matching(tree, pattern)?;
        }

        let out_string = tree.to_newick();
        writer.write_all((out_string + "\n").as_ref())?;
    }

    writer.flush()?;
    Ok(())
}
