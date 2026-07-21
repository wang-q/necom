use anyhow::Context;
use clap::{ArgMatches, Command};
use std::io::Write;

/// Build the clap subcommand for k-medoids.
pub fn make_subcommand() -> Command {
    Command::new("k-medoids")
        .about("Clusters entries via K-Medoids")
        .visible_alias("km")
        .after_help(include_str!("../../../docs/help/clust/k-medoids.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input file containing pairwise distances in .tsv format. [stdin] for standard input",
        ))
        .arg(crate::cmd_necom::args::k_arg().required(true))
        .arg(crate::cmd_necom::args::format_arg())
        .arg(crate::cmd_necom::args::flat_rep_arg())
        .arg(crate::cmd_necom::args::same_arg("0.0"))
        .arg(crate::cmd_necom::args::missing_arg("1.0"))
        .arg(crate::cmd_necom::args::runs_arg())
        .arg(crate::cmd_necom::args::max_iter_arg())
        .arg(crate::cmd_necom::args::seed_arg(
            None,
            None,
            "Random seed for reproducible initialization",
        ))
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Execute the k-medoids command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // 1. Args
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let opt_k = *args
        .get_one::<usize>("k")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: k"))?;
    // Remaining arguments have clap default values, so unwrap is safe.
    let opt_format = args.get_one::<String>("clust_format").unwrap();
    let opt_rep = args.get_one::<String>("flat_rep").unwrap().as_str();
    let opt_same = *args.get_one::<f32>("same").unwrap();
    let opt_missing = *args.get_one::<f32>("missing").unwrap();
    let runs = *args.get_one::<usize>("runs").unwrap();
    let max_iter = *args.get_one::<usize>("max_iter").unwrap();
    if opt_k == 0 {
        anyhow::bail!("--k must be greater than 0");
    }
    if runs == 0 {
        anyhow::bail!("--runs must be greater than 0");
    }
    if max_iter == 0 {
        anyhow::bail!("--max-iter must be greater than 0");
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

    if opt_k > names.len() {
        anyhow::bail!(
            "--k ({}) cannot exceed the number of samples ({})",
            opt_k,
            names.len()
        );
    }

    // 3. Clustering
    let mut kmedoids =
        necom::libs::clust::k_medoids::KMedoids::new(opt_k, max_iter, runs);
    if let Some(&seed) = args.get_one::<u64>("seed") {
        kmedoids = kmedoids.with_seed(seed);
    }
    let mut clusters = kmedoids.perform_clustering(&matrix);

    // 4. Output
    let out = necom::libs::clust::format::format_flat_clusters_with_rep(
        &mut clusters,
        &names,
        &matrix,
        opt_rep,
        false,
        opt_format,
    )?;

    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;
    writer.write_all(out.as_bytes())?;

    writer.flush()?;
    Ok(())
}
