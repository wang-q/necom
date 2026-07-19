use anyhow::Context;
use clap::{ArgMatches, Command};
use std::io::Write;
use std::str::FromStr;

use necom::libs::clust::hier::{linkage_inplace, to_tree, Method};
use necom::libs::pairmat::NamedMatrix;

/// Build the clap subcommand for hier.
pub fn make_subcommand() -> Command {
    Command::new("hier")
        .about("Clusters entries via hierarchical clustering")
        .visible_alias("hclust")
        .after_help(include_str!("../../../docs/help/clust/hier.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input PHYLIP distance matrix file. [stdin] for standard input",
        ))
        .arg(crate::cmd_necom::args::clust_method_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Execute the hier command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    // clust_method has a clap default value, so unwrap is safe.
    let method_str = args.get_one::<String>("clust_method").unwrap();
    let outfile = crate::cmd_necom::args::get_outfile(args);

    // Parse method
    let method = Method::from_str(method_str)
        .with_context(|| format!("invalid --clust-method '{}'", method_str))?;

    // Read matrix
    let matrix = NamedMatrix::from_relaxed_phylip(infile)?;

    // Perform clustering
    let (names, condensed) = matrix.into_parts();
    let steps = linkage_inplace(condensed, method);

    // Convert to tree
    let tree = to_tree(&steps, &names)?;

    // Format output
    let newick = tree.to_newick();

    // Write output
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;
    writer.write_all((newick + "\n").as_ref())?;

    writer.flush()?;
    Ok(())
}
