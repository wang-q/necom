use anyhow::Context;
use clap::{builder, Arg, ArgMatches, Command};
use std::io::Write;

use crate::cmd_necom::clust::dbscan::resolve_min_points;
use necom::libs::eval::{davies_bouldin_distance_score, silhouette_score, LabelMap};

/// Build the `clust scan-dbscan` subcommand.
pub fn make_subcommand() -> Command {
    Command::new("scan-dbscan")
        .about("Scans DBSCAN epsilon and reports internal metrics")
        .after_help(include_str!("../../../docs/help/clust/scan-dbscan.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input file containing pairwise distances in .tsv format. [stdin] for standard input",
        ))
        .arg(crate::cmd_necom::args::format_arg())
        .arg(crate::cmd_necom::args::flat_rep_arg())
        .arg(crate::cmd_necom::args::same_arg("0.0"))
        .arg(crate::cmd_necom::args::missing_arg("1.0"))
        .arg(crate::cmd_necom::args::min_points_arg())
        .arg(crate::cmd_necom::clust::dbscan::min_pct_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
        .arg(
            Arg::new("scan")
                .long("scan")
                .num_args(1)
                .required(true)
                .help("Scan range for eps (format: start,end,step)"),
        )
        .arg(
            Arg::new("opt_eps")
                .long("opt-eps")
                .num_args(1)
                .value_parser([
                    builder::PossibleValue::new("silhouette"),
                    builder::PossibleValue::new("max-clusters"),
                    builder::PossibleValue::new("min-noise"),
                ])
                .help("Select the best eps and output that partition instead of the summary"),
        )
}

/// Execute the `clust scan-dbscan` command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // 1. Args
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;

    let opt_format = args.get_one::<String>("clust_format").unwrap();
    let opt_rep = args.get_one::<String>("flat_rep").unwrap().as_str();
    let opt_same = *args.get_one::<f32>("same").unwrap();
    let opt_missing = *args.get_one::<f32>("missing").unwrap();
    let opt_eps = args.get_one::<String>("opt_eps").map(|s| s.as_str());

    if !opt_same.is_finite() {
        anyhow::bail!("--same must be a finite number, got {}", opt_same);
    }
    if !opt_missing.is_finite() {
        anyhow::bail!("--missing must be a finite number, got {}", opt_missing);
    }

    let scan_range = args.get_one::<String>("scan").unwrap();
    let (start, end, step) = necom::libs::cut::scan::parse_scan_range(scan_range)?;
    if start <= 0.0 {
        anyhow::bail!("--scan start must be a positive number, got {}", start);
    }

    let outfile = crate::cmd_necom::args::get_outfile(args);

    // 2. Load Matrix
    let matrix = necom::libs::pairmat::NamedMatrix::from_pair_scores(
        infile,
        opt_same,
        opt_missing,
    )?;
    let names: Vec<String> = matrix.get_names().iter().map(|s| s.to_string()).collect();
    let opt_min_points = resolve_min_points(args, names.len())?;

    // 3. Build eps values
    let eps_values = build_eps_values(start, end, step)?;

    // 4. Scan
    let mut best_eps: Option<f32> = None;
    let mut best_score: Option<f64> = None;

    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    if opt_eps.is_none() {
        writer.write_all(b"Epsilon\tClusters\tNoise\tSilhouette\tDBIndex\n")?;
    }

    for eps in &eps_values {
        let mut dbscan = necom::libs::clust::dbscan::Dbscan::new(*eps, opt_min_points)?;
        dbscan.perform_clustering(&matrix);
        let (cluster_count, noise_count) = dbscan.counts();
        let clusters = dbscan.results_cluster();

        let partition = clusters_to_partition(&clusters, &names);
        let silhouette = silhouette_score(&partition, &matrix);
        let db_index = davies_bouldin_distance_score(&partition, &matrix);

        if let Some(criterion) = opt_eps {
            let score = match criterion {
                "silhouette" => {
                    if silhouette.is_nan() {
                        f64::NEG_INFINITY
                    } else {
                        silhouette
                    }
                }
                "max-clusters" => cluster_count as f64,
                "min-noise" => -(noise_count as f64),
                _ => unreachable!(),
            };

            let is_better = match best_score {
                None => true,
                Some(current) => {
                    // Prefer smaller eps on ties.
                    score > current || (score == current && Some(*eps) < best_eps)
                }
            };

            if is_better {
                best_score = Some(score);
                best_eps = Some(*eps);
            }
        } else {
            writer.write_all(
                format!(
                    "{}\t{}\t{}\t{}\t{}\n",
                    format_eps(*eps),
                    cluster_count,
                    noise_count,
                    format_metric(silhouette),
                    format_metric(db_index)
                )
                .as_bytes(),
            )?;
        }
    }

    // 5. Output the best partition if requested.
    if let Some(criterion) = opt_eps {
        let eps = best_eps.ok_or_else(|| {
            anyhow::anyhow!("could not select a best eps for criterion '{}'", criterion)
        })?;

        let mut dbscan = necom::libs::clust::dbscan::Dbscan::new(eps, opt_min_points)?;
        dbscan.perform_clustering(&matrix);
        let mut clusters = dbscan.results_cluster();

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
    }

    writer.flush()?;
    Ok(())
}

/// Build the list of eps values from a validated scan range.
fn build_eps_values(start: f64, end: f64, step: f64) -> anyhow::Result<Vec<f32>> {
    let n_steps_f = ((end - start) / step).floor();
    if !n_steps_f.is_finite() || n_steps_f < 0.0 || n_steps_f > i64::MAX as f64 {
        anyhow::bail!(
            "scan range too large: start={}, end={}, step={}",
            start,
            end,
            step
        );
    }
    let n_steps = n_steps_f as i64;

    let mut values: Vec<f32> = Vec::with_capacity((n_steps + 2) as usize);
    for i in 0..=n_steps {
        values.push((start + (i as f64) * step) as f32);
    }
    // Ensure the explicit end value is included even when step does not
    // evenly divide the interval.
    if let Some(&last) = values.last() {
        if end > (last as f64) + 1e-9 {
            values.push(end as f32);
        }
    }
    Ok(values)
}

/// Convert flat clustering results (noise points emitted as singletons) into a
/// partition map suitable for `libs::eval` metrics.
fn clusters_to_partition(clusters: &[Vec<usize>], names: &[String]) -> LabelMap {
    let mut partition = LabelMap::new();
    for (label, members) in clusters.iter().enumerate() {
        for &idx in members {
            partition.insert(names[idx].clone(), label as u32);
        }
    }
    partition
}

/// Format a metric value, emitting `NA` for non-finite values.
fn format_metric(value: f64) -> String {
    if value.is_finite() {
        format!("{}", value)
    } else {
        "NA".to_string()
    }
}

/// Format an eps value for output, stripping floating-point drift.
fn format_eps(value: f32) -> String {
    let s = format!("{:.10}", value);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
