use anyhow::Context;
use clap::{ArgMatches, Command};
use std::io::Write;

/// Build the clap subcommand for to-pair.
pub fn make_subcommand() -> Command {
    Command::new("to-pair")
        .about("Converts a PHYLIP distance matrix to pairwise distances")
        .after_help(include_str!("../../../docs/help/mat/to-pair.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input file containing a PHYLIP distance matrix",
        ))
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the to-pair command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args.get_one::<String>("infile").unwrap();
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    // Load matrix from PHYLIP format
    let matrix = necom::libs::pairmat::NamedMatrix::from_relaxed_phylip(infile)?;
    let names = matrix.get_names();

    // Output pairwise distances (lower triangle only)
    for i in 0..matrix.size() {
        for j in 0..=i {
            let distance = matrix.get(i, j);
            writer
                .write_fmt(format_args!("{}\t{}\t{}\n", names[j], names[i], distance))?;
        }
    }

    writer.flush()?;
    Ok(())
}
