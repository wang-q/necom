use anyhow::Context;
use clap::{builder::PossibleValue, Arg, ArgAction, ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
use std::collections::BTreeSet;
use std::io::Write;

/// Build the clap subcommand for label.
pub fn make_subcommand() -> Command {
    Command::new("label")
        .about("Extracts labels from a Newick file")
        .after_help(
            r###"
Extracts the tree's labels.

Notes:
* By default, prints all labels that occur in the tree, in the same order as
  in the Newick, one per line. Empty labels produce no output.
* The `--tab` option prints labels on a single line, separated by tabs.
* The `-I` and `-L` options filter out internal or leaf nodes.
* Selection options (`-n`, `-l`, `-x`) can be combined.
* With `-D`, descendants of selected internal nodes are also included.
* Clade check (`-M`) verifies if the selected nodes form a monophyletic group
  with at least two nodes. It checks terminal nodes against the selection.
* Warning: Duplicate node names may affect selection/clade checks.
* Extra columns (`-c`) details:
    * `dup`: duplicate the node name
    * `taxid`: `:T=` field in comment
    * `species`: `:S=` field in comment
    * `full`: full comment

Examples:
1. List all labels:
   necom nwk label tree.nwk

2. Count leaves:
   necom nwk label tree.nwk -I | wc -l

3. List specific nodes:
   necom nwk label tree.nwk -n Human -n Chimp

4. List labels matching regex:
   necom nwk label tree.nwk -x "^Homo"

5. Check clade:
   necom nwk label tree.nwk -n Human -n Chimp -M

"###,
        )
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::internal_arg())
        .arg(crate::cmd_necom::args::leaf_arg())
        .arg(crate::cmd_necom::args::node_arg())
        .arg(crate::cmd_necom::args::name_list_arg())
        .arg(crate::cmd_necom::args::regex_arg())
        .arg(crate::cmd_necom::args::descendants_arg())
        .arg(
            Arg::new("root")
                .long("root")
                .action(ArgAction::SetTrue)
                .help("Only print the root label"),
        )
        .arg(
            Arg::new("tab")
                .long("tab")
                .action(ArgAction::SetTrue)
                .help("Print labels on a single line, separated by tab stops"),
        )
        .arg(crate::cmd_necom::args::monophyly_arg(
            "Only print the labels when they form a clade",
        ))
        .arg(
            Arg::new("extra_column")
                .long("extra-column")
                .short('c')
                .action(ArgAction::Append)
                .value_parser([
                    PossibleValue::new("dup"),
                    PossibleValue::new("taxid"),
                    PossibleValue::new("species"),
                    PossibleValue::new("full"),
                ])
                .help("Add extra columns to output"),
        )
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the label command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer =
        necom::writer(outfile).with_context(|| format!("Failed to open writer for {}", outfile))?;

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let trees = Tree::from_file(infile)?;
    if trees.is_empty() {
        return Ok(());
    }
    let mut columns = vec![];
    if args.contains_id("extra_column") {
        for column in args
            .get_many::<String>("extra_column")
            .ok_or_else(|| anyhow::anyhow!("missing required argument: extra_column"))?
        {
            columns.push(column.to_string());
        }
    }

    let tab_sep = args.get_flag("tab");

    for tree in &trees {
        let mut collected_labels = Vec::new();

        // Handle --root option
        if args.get_flag("root") {
            let root_id = tree.get_root().context("tree has no root")?;
            let root = tree.get_node(root_id).context("root node not found")?;
            if let Some(name) = &root.name {
                if !name.is_empty() {
                    let out_string = super::common::format_label_columns(root, name, &columns);
                    if tab_sep {
                        collected_labels.push(out_string);
                    } else {
                        writer.write_fmt(format_args!("{}\n", out_string))?;
                    }
                }
            }
            if tab_sep && !collected_labels.is_empty() {
                writer.write_fmt(format_args!("{}\n", collected_labels.join("\t")))?;
            }
            continue;
        }

        let is_monophyly = args.get_flag("monophyly");

        // Operating
        // All IDs matching positions
        let ids_pos = super::common::match_positions(tree, args)?;

        // All IDs matching names
        let ids_name = super::common::match_names(tree, args)?;

        let ids: BTreeSet<usize> = ids_pos.intersection(&ids_name).cloned().collect();

        // Print nothing if the selected nodes do not form a clade.
        let ids_vec: Vec<usize> = ids.iter().cloned().collect();
        if is_monophyly && !tree.is_clade(&ids_vec) {
            continue;
        }

        for id in ids.iter() {
            let node = tree.get_node(*id).context("node not found")?;
            if let Some(x) = node.name.clone() {
                if !x.is_empty() {
                    let out_string = super::common::format_label_columns(node, &x, &columns);

                    if tab_sep {
                        collected_labels.push(out_string);
                    } else {
                        writer.write_fmt(format_args!("{}\n", out_string))?;
                    }
                }
            }
        }

        if tab_sep && !collected_labels.is_empty() {
            writer.write_fmt(format_args!("{}\n", collected_labels.join("\t")))?;
        }
    }

    writer.flush()?;
    Ok(())
}
