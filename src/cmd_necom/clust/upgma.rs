use anyhow::Context;
use clap::{ArgMatches, Command};
use necom::libs::clust::upgma;
use std::io::Write;

/// Build the clap subcommand for upgma.
pub fn make_subcommand() -> Command {
    Command::new("upgma")
        .about("Constructs a phylogenetic tree using UPGMA")
        .after_help(include_str!("../../../docs/help/clust/upgma.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input PHYLIP distance matrix file. [stdin] for standard input",
        ))
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Execute the upgma command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let outfile = crate::cmd_necom::args::get_outfile(args);

    // Load matrix
    let matrix = necom::libs::pairmat::NamedMatrix::from_relaxed_phylip(infile)?;

    // Build tree
    let tree = upgma::upgma(&matrix)?;

    // Output tree
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;
    writer.write_all((tree.to_newick() + "\n").as_ref())?;

    writer.flush()?;
    Ok(())
}
