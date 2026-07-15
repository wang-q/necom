use anyhow::Context;
use clap::{ArgMatches, Command};
use necom::libs::clust::nj;
use std::io::Write;

/// Build the clap subcommand for nj.
pub fn make_subcommand() -> Command {
    Command::new("nj")
        .about("Constructs a phylogenetic tree using Neighbor-Joining")
        .after_help(
            r###"
Constructs a phylogenetic tree from a distance matrix using the Neighbor-Joining (NJ) algorithm.

Notes:
* Input: PHYLIP distance matrix (relaxed or strict).
* Output: Newick tree (midpoint rooted).
* NJ is a bottom-up clustering method suitable for variable evolutionary rates.

Examples:
1. Build tree from matrix:
   necom clust nj matrix.phy -o tree.nwk

2. Pipe matrix to tree:
   cat matrix.phy | necom clust nj stdin > tree.nwk
"###,
        )
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input PHYLIP matrix file. [stdin] for standard input",
        ))
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Execute the nj command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let outfile = crate::cmd_necom::args::get_outfile(args);

    // Load matrix
    let matrix = necom::libs::pairmat::NamedMatrix::from_relaxed_phylip(infile)?;

    // Build tree
    let tree = nj::nj(&matrix)?;

    // Output tree
    let mut writer =
        necom::writer(outfile).with_context(|| format!("Failed to open writer for {}", outfile))?;
    writer.write_all((tree.to_newick() + "\n").as_ref())?;

    writer.flush()?;
    Ok(())
}
