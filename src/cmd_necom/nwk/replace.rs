use anyhow::Context;
use clap::{ArgMatches, Command};
use necom::libs::phylo::tree::{AnnotationMode, Tree};
use std::io::Write;

/// Build the clap subcommand for replace.
pub fn make_subcommand() -> Command {
    Command::new("replace")
        .about("Replaces node names or comments in a Newick file")
        .after_help(include_str!("../../../docs/help/nwk/replace.md"))
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

    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    for tree in &mut trees {
        tree.replace_annotations(
            annotation_mode,
            &replace_map,
            skip_internal,
            skip_leaf,
        )?;

        let out_string = tree.to_newick();
        writer.write_fmt(format_args!("{}\n", out_string))?;
    }

    writer.flush()?;
    Ok(())
}
