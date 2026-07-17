use anyhow::Context;
use clap::{ArgMatches, Command};
use std::io::Write;
/// Build the clap subcommand for subset.
pub fn make_subcommand() -> Command {
    Command::new("subset")
        .about("Extracts a submatrix from a PHYLIP matrix using a list of names")
        .after_help(include_str!("../../../docs/help/mat/subset.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input PHYLIP matrix file",
        ))
        .arg(crate::cmd_necom::args::mat_name_list_arg(true))
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Execute the subset command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args.get_one::<String>("infile").unwrap();
    let list_file = args.get_one::<String>("name_list").unwrap();
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    let wanted_names = necom::libs::io::read_names::<Vec<String>>(list_file)?;

    // Load and process matrix
    let matrix = necom::libs::pairmat::NamedMatrix::from_relaxed_phylip(infile)?;

    let missing =
        necom::libs::pairmat::write_subset(&matrix, &wanted_names, &mut writer)?;
    for name in &missing {
        log::warn!("Name not found in matrix: {}", name);
    }

    writer.flush()?;
    Ok(())
}
