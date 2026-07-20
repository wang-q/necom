use anyhow::Context;
use clap::{Arg, ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
use std::io::Write;

/// Build the clap subcommand for to-svg.
pub fn make_subcommand() -> Command {
    Command::new("to-svg")
        .about("Converts Newick trees to SVG format")
        .after_help(include_str!("../../../docs/help/nwk/to-svg.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(
            Arg::new("width")
                .short('w')
                .long("width")
                .num_args(1)
                .default_value("800")
                .value_parser(clap::value_parser!(f64))
                .help("SVG width in pixels"),
        )
        .arg(
            Arg::new("vskip")
                .short('v')
                .long("vskip")
                .num_args(1)
                .default_value("20")
                .value_parser(clap::value_parser!(f64))
                .help("Vertical spacing between leaf nodes in pixels"),
        )
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the to-svg command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let width: f64 = *args
        .get_one::<f64>("width")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: width"))?;
    if width <= 0.0 || !width.is_finite() {
        anyhow::bail!("--width must be a positive finite number");
    }

    let vskip: f64 = *args
        .get_one::<f64>("vskip")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: vskip"))?;
    if vskip <= 0.0 || !vskip.is_finite() {
        anyhow::bail!("--vskip must be a positive finite number");
    }

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
    let tree = trees
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("no trees found in {}", infile))?;

    // Auto-detect: if any non-root node has a branch length, draw phylogram.
    let has_bl = necom::libs::phylo::tree::stat::has_branch_lengths(&tree);
    let height = super::common::display_height(&tree, has_bl);

    let out_string = necom::libs::phylo::tree::io::to_svg(&tree, height, vskip, width);

    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    writer.write_all(out_string.as_ref())?;

    writer.flush()?;
    Ok(())
}
