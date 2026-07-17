use anyhow::Context;
use clap::{ArgMatches, Command};
use std::io::Write;

/// Build the clap subcommand for cc.
pub fn make_subcommand() -> Command {
    Command::new("cc")
        .about("Clusters entries via connected components")
        .after_help(include_str!("../../../docs/help/clust/cc.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input file containing pairwise relations (weights ignored) in .tsv format",
        ))
        .arg(crate::cmd_necom::args::format_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the cc command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // 1. Args
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    // clust_format has a clap default value, so unwrap is safe.
    let opt_format = args.get_one::<String>("clust_format").unwrap();
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    // 2. Load Graph & Clustering
    let reader = necom::reader(infile)
        .with_context(|| format!("Failed to open reader for {}", infile))?;
    let (names_vec, mut scc) = necom::libs::clust::connected_components(reader)?;

    // 3. Output
    let out = necom::libs::clust::format::format_flat_clusters(
        &mut scc,
        &names_vec,
        opt_format,
        |c| c.first().copied(),
    )?;
    writer.write_all(out.as_bytes())?;

    writer.flush()?;
    Ok(())
}
