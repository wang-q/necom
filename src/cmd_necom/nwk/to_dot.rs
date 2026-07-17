use anyhow::Context;
use clap::{ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
use std::io::Write;

/// Build the clap subcommand for to-dot.
pub fn make_subcommand() -> Command {
    Command::new("to-dot")
        .about("Converts Newick trees to Graphviz DOT format")
        .after_help(include_str!("../../../docs/help/nwk/to-dot.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the to-dot command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let trees = Tree::from_file(infile)?;

    for tree in trees {
        let out_string = tree.to_dot();
        writer.write_fmt(format_args!("{}\n", out_string))?;
    }

    writer.flush()?;
    Ok(())
}
