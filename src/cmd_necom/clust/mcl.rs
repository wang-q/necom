use anyhow::Context;
use clap::{ArgMatches, Command};
use std::io::Write;

/// Build the clap subcommand for mcl.
pub fn make_subcommand() -> Command {
    Command::new("mcl")
        .about("Clusters entries via Markov Clustering (MCL)")
        .after_help(
            r###"
MCL is a fast and scalable unsupervised cluster algorithm for graphs (also known as networks) based on simulation of (stochastic) flow in graphs.

It is particularly useful for clustering protein interaction networks or similarity networks.

Note: The input file should contain similarity scores (higher is better), NOT distances.

Output formats:
    * cluster: Each line contains points of one cluster.
    * pair: Each line contains a (representative point, cluster member) pair.

Note:
The representative point is selected by --rep and affects both output formats:
    * For 'pair' format: it is the first column (representative ID).
    * For 'cluster' format: it is placed in the first column.
    * medoid (default): point with maximum sum of similarities to other cluster members.
    * first: alphabetically first member of the cluster.

Reference:
Stijn van Dongen, Graph Clustering by Flow Simulation. PhD thesis, University of Utrecht, May 2000.
"###,
        )
        .arg(
            crate::cmd_necom::args::infile_arg_required_with_help(
                "Input file containing pairwise similarities (edge weights) in .tsv format",
            ),
        )
        .arg(crate::cmd_necom::args::format_arg())
        .arg(crate::cmd_necom::args::flat_rep_arg())
        .arg(crate::cmd_necom::args::same_arg("1.0"))
        .arg(crate::cmd_necom::args::missing_arg("0.0"))
        .arg(crate::cmd_necom::args::mcl_inflation_arg())
        .arg(crate::cmd_necom::args::mcl_prune_arg())
        .arg(crate::cmd_necom::args::max_iter_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Execute the mcl command.
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
    let inflation = *args.get_one::<f64>("inflation").unwrap();
    let prune = *args.get_one::<f64>("prune").unwrap();
    let max_iter = *args.get_one::<usize>("max_iter").unwrap();
    let outfile = crate::cmd_necom::args::get_outfile(args);

    let mut writer =
        necom::writer(outfile).with_context(|| format!("Failed to open writer for {}", outfile))?;

    // 2. Load Matrix
    // ScoringMatrix::from_pair_scores is only implemented for f32
    let (sm, names) = necom::libs::pairmat::ScoringMatrix::<f32>::from_pair_scores(
        infile,
        opt_same,
        opt_missing,
    )?;

    // 3. Clustering
    let mut mcl = necom::libs::clust::mcl::Mcl::new(inflation);
    mcl.set_prune_limit(prune);
    mcl.set_max_iter(max_iter);
    let mut clusters = mcl.perform_clustering(&sm);

    // 4. Output
    let out = if opt_rep == "first" {
        necom::libs::clust::format::format_flat_clusters(&mut clusters, &names, opt_format, |c| {
            c.first().copied()
        })?
    } else {
        necom::libs::clust::format::format_flat_clusters(&mut clusters, &names, opt_format, |c| {
            necom::libs::clust::medoid::find_medoid(&sm, c, true)
        })?
    };
    writer.write_all(out.as_bytes())?;

    writer.flush()?;
    Ok(())
}
