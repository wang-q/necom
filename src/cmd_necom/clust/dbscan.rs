use anyhow::Context;
use clap::{ArgMatches, Command};
use std::io::Write;

/// Build the clap subcommand for dbscan.
pub fn make_subcommand() -> Command {
    Command::new("dbscan")
        .about("Clusters entries via DBSCAN")
        .after_help(include_str!("../../../docs/help/clust/dbscan.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input file containing pairwise distances in .tsv format",
        ))
        .arg(crate::cmd_necom::args::format_arg())
        .arg(crate::cmd_necom::args::flat_rep_arg())
        .arg(crate::cmd_necom::args::same_arg("0.0"))
        .arg(crate::cmd_necom::args::missing_arg("1.0"))
        .arg(crate::cmd_necom::args::eps_arg())
        .arg(crate::cmd_necom::args::min_points_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the dbscan command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // 1. Args
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;

    // Remaining arguments have clap default values, so unwrap is safe.
    let opt_format = args.get_one::<String>("clust_format").unwrap();
    let opt_rep = args.get_one::<String>("flat_rep").unwrap().as_str();
    let opt_same = *args.get_one::<f32>("same").unwrap();
    let opt_missing = *args.get_one::<f32>("missing").unwrap();
    let opt_eps = *args.get_one::<f32>("eps").unwrap();
    let opt_min_points = *args.get_one::<usize>("min_points").unwrap();

    if opt_eps <= 0.0 {
        anyhow::bail!("--eps must be positive, got {}", opt_eps);
    }
    if opt_min_points == 0 {
        anyhow::bail!("--min-points must be at least 1");
    }

    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    // 2. Load Matrix

    // Load matrix from pairwise distances
    let (matrix, names) = necom::libs::pairmat::ScoringMatrix::from_pair_scores(
        infile,
        opt_same,
        opt_missing,
    )?;

    // 3. Clustering
    let mut dbscan = necom::libs::clust::dbscan::Dbscan::new(opt_eps, opt_min_points);
    dbscan.perform_clustering(&matrix);
    let mut clusters = dbscan.results_cluster();

    // 4. Output
    let out = if opt_rep == "first" {
        necom::libs::clust::format::format_flat_clusters(
            &mut clusters,
            &names,
            opt_format,
            |c| c.first().copied(),
        )?
    } else {
        necom::libs::clust::format::format_flat_clusters(
            &mut clusters,
            &names,
            opt_format,
            |c| necom::libs::clust::medoid::find_medoid(&matrix, c, false),
        )?
    };
    writer.write_all(out.as_bytes())?;

    writer.flush()?;
    Ok(())
}
