use anyhow::Context;
use clap::{ArgMatches, Command};
use std::io::Write;
/// Build the clap subcommand for format.
pub fn make_subcommand() -> Command {
    Command::new("format")
        .about("Converts between different PHYLIP matrix formats")
        .after_help(include_str!("../../../docs/help/mat/format.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input PHYLIP matrix file",
        ))
        .arg(crate::cmd_necom::args::mat_format_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Execute the format command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args.get_one::<String>("infile").unwrap();
    let opt_mode = args.get_one::<String>("mat_format").unwrap();
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer =
        necom::writer(outfile).with_context(|| format!("Failed to open writer for {}", outfile))?;

    let matrix = necom::libs::pairmat::NamedMatrix::from_relaxed_phylip(infile)?;
    let fmt = necom::libs::pairmat::MatrixFormat::from_mode(opt_mode)?;

    necom::libs::pairmat::write_phylip_matrix(&matrix, fmt, None, &mut writer)?;

    writer.flush()?;
    Ok(())
}
