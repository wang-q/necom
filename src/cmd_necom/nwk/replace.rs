use anyhow::Context;
use clap::{ArgMatches, Command};
use necom::libs::phylo::tree::{AnnotationMode, Tree};
use std::io::Write;

/// Build the clap subcommand for replace.
pub fn make_subcommand() -> Command {
    Command::new("replace")
        .about("Replaces node names or comments in a Newick file")
        .after_help(
            r###"
Replaces node names or appends annotations in a Newick file using a TSV file.

Notes:
* `--replace-tsv` is a tab-separated file with 2 or more columns:
  `<original_name> <replacement> [additional_annotations...]`
* The behavior of the 2nd column (`<replacement>`) depends on `--mode`:
    * `label` (default): Replaces the node name. Empty string removes the name.
    * `taxid`:           Appends as NCBI TaxID (`:T=<replacement>`) in NHX.
    * `species`:         Appends as species name (`:S=<replacement>`) in NHX.
    * `asis`:            Appends as comments/properties. Values containing `=` are parsed as `key=value` pairs; bare values are stored as keys with empty values.
* Columns 3+ are ALWAYS appended to the node's comments/properties.
  Key-value pairs (e.g., `color=red`) are stored as properties.
  Simple tags (e.g., `highlight`) are stored as keys with empty values.

Examples:
1. Basic renaming of nodes:
   necom nwk replace input.nwk --replace-tsv names.tsv > output.nwk

2. Add species and color annotations:
   necom nwk replace input.nwk --replace-tsv annotations.tsv --mode species

3. Remove node names (2nd column is empty):
   necom nwk replace input.nwk --replace-tsv remove.tsv

"###,
        )
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::replace_tsv_arg())
        .arg(crate::cmd_necom::args::internal_arg())
        .arg(crate::cmd_necom::args::leaf_arg())
        .arg(crate::cmd_necom::args::mode_arg(
            "label",
            &["label", "taxid", "species", "asis"],
            "Where we place the replaces",
        ))
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the replace command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer =
        necom::writer(outfile).with_context(|| format!("Failed to open writer for {}", outfile))?;

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let mode = args
        .get_one::<String>("mode")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: mode"))?;
    let annotation_mode = match mode.as_str() {
        "label" => AnnotationMode::Label,
        "taxid" => AnnotationMode::TaxId,
        "species" => AnnotationMode::Species,
        "asis" => AnnotationMode::AsIs,
        other => anyhow::bail!("unknown property mode: {}", other),
    };

    let skip_internal = args.get_flag("internal");
    let skip_leaf = args.get_flag("leaf");

    let rfile = args
        .get_one::<String>("replace_tsv")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: replace_tsv"))?;
    let replace_map = necom::libs::io::read_replace_tsv_overwrite(rfile)?;

    let mut trees = Tree::from_file(infile)?;

    for tree in &mut trees {
        tree.replace_annotations(annotation_mode, &replace_map, skip_internal, skip_leaf)?;

        let out_string = tree.to_newick();
        writer.write_all((out_string + "\n").as_ref())?;
    }

    writer.flush()?;
    Ok(())
}
