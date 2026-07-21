use anyhow::Context;
use clap::parser::ValueSource;
use clap::{Arg, ArgMatches, Command};
use std::io::Write;

/// Build the clap subcommand for dbscan.
pub fn make_subcommand() -> Command {
    Command::new("dbscan")
        .about("Clusters entries via DBSCAN")
        .after_help(include_str!("../../../docs/help/clust/dbscan.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input file containing pairwise distances in .tsv format. [stdin] for standard input",
        ))
        .arg(crate::cmd_necom::args::format_arg())
        .arg(crate::cmd_necom::args::flat_rep_arg())
        .arg(crate::cmd_necom::args::same_arg("0.0"))
        .arg(crate::cmd_necom::args::missing_arg("1.0"))
        .arg(crate::cmd_necom::args::eps_arg())
        .arg(crate::cmd_necom::args::min_points_arg())
        .arg(min_pct_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// `--min-pct` argument: alternative way to specify `min_points` as a fraction
/// of the total number of samples.
pub fn min_pct_arg() -> Arg {
    Arg::new("min_pct")
        .long("min-pct")
        .num_args(1)
        .value_parser(clap::value_parser!(f64))
        .help("Minimum points as a fraction of total samples (0..1). Alternative to --min-points")
}

/// Resolve the effective `min_points` value from `--min-points` and `--min-pct`.
///
/// The two options are mutually exclusive. If `--min-pct` is provided, it is
/// converted to an absolute count via `ceil(pct * n_samples)`.
pub fn resolve_min_points(args: &ArgMatches, n_samples: usize) -> anyhow::Result<usize> {
    let min_points_user =
        args.value_source("min_points") == Some(ValueSource::CommandLine);
    let min_pct_user = args.value_source("min_pct") == Some(ValueSource::CommandLine);

    if min_points_user && min_pct_user {
        anyhow::bail!("--min-points and --min-pct are mutually exclusive");
    }

    let resolved = if let Some(&pct) = args.get_one::<f64>("min_pct") {
        if !pct.is_finite() || pct <= 0.0 || pct > 1.0 {
            anyhow::bail!("--min-pct must be in (0, 1], got {}", pct);
        }
        let computed = (pct * n_samples as f64).ceil() as usize;
        if computed < 1 {
            anyhow::bail!("--min-pct {} is too small for {} samples", pct, n_samples);
        }
        computed
    } else {
        *args.get_one::<usize>("min_points").unwrap()
    };

    if resolved < 1 {
        anyhow::bail!("--min-points must be at least 1");
    }

    Ok(resolved)
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

    if !opt_eps.is_finite() || opt_eps <= 0.0 {
        anyhow::bail!("--eps must be a positive finite number, got {}", opt_eps);
    }
    if !opt_same.is_finite() {
        anyhow::bail!("--same must be a finite number, got {}", opt_same);
    }
    if !opt_missing.is_finite() {
        anyhow::bail!("--missing must be a finite number, got {}", opt_missing);
    }

    let outfile = crate::cmd_necom::args::get_outfile(args);

    // 2. Load Matrix (before opening writer so input failures do not
    //    truncate the output file).
    let matrix = necom::libs::pairmat::NamedMatrix::from_pair_scores(
        infile,
        opt_same,
        opt_missing,
    )?;
    let names: Vec<String> = matrix.get_names().iter().map(|s| s.to_string()).collect();
    let opt_min_points = resolve_min_points(args, names.len())?;

    // 3. Clustering
    let mut dbscan = necom::libs::clust::dbscan::Dbscan::new(opt_eps, opt_min_points)?;
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

    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;
    writer.write_all(out.as_bytes())?;

    writer.flush()?;
    Ok(())
}
