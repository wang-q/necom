use anyhow::Context;
use clap::{Arg, ArgMatches, Command};
use necom::libs::phylo::tree::{stat, Tree};
use std::io::Write;

/// Build the clap subcommand for stat.
pub fn make_subcommand() -> Command {
    Command::new("stat")
        .about("Prints statistics about trees")
        .after_help(include_str!("../../../docs/help/nwk/stat.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::outfile_arg())
        .arg(
            Arg::new("style")
                .long("style")
                .value_parser(["col", "line"])
                .default_value("col")
                .help("Output style. [col] for key-value pairs, [line] for TSV"),
        )
}

/// Execute the stat command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let style = args
        .get_one::<String>("style")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: style"))?;

    let trees = Tree::from_file(infile)?;

    if style == "line" {
        writer.write_fmt(format_args!("{}\n", stat::TreeSummary::tsv_header()))?;
    }

    for tree in trees {
        let s = stat::tree_summary(&tree);

        if style == "line" {
            writer.write_fmt(format_args!("{}\n", s.to_tsv_line()))?;
        } else {
            for line in s.to_kv_lines() {
                writer.write_fmt(format_args!("{}\n", line))?;
            }
        }
    }

    writer.flush()?;
    Ok(())
}
