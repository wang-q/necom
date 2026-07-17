use anyhow::Context;
use clap::{ArgMatches, Command};

/// Build the clap subcommand for to-phylip.
pub fn make_subcommand() -> Command {
    Command::new("to-phylip")
        .about("Converts pairwise distances to a phylip distance matrix")
        .after_help(include_str!("../../../docs/help/mat/to-phylip.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input file containing pairwise distances",
        ))
        .arg(crate::cmd_necom::args::same_arg("0.0"))
        .arg(crate::cmd_necom::args::missing_arg("1.0"))
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the to-phylip command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args.get_one::<String>("infile").unwrap();
    let opt_same = *args.get_one::<f32>("same").unwrap();
    let opt_missing = *args.get_one::<f32>("missing").unwrap();
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    // Load matrix from pairwise distances
    let matrix = necom::libs::pairmat::NamedMatrix::from_pair_scores(
        infile,
        opt_same,
        opt_missing,
    )?;

    necom::libs::pairmat::write_phylip_matrix(
        &matrix,
        necom::libs::pairmat::MatrixFormat::Full,
        None,
        &mut writer,
    )?;

    writer.flush()?;
    Ok(())
}
