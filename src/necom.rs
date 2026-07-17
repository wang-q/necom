use clap::{crate_authors, crate_version, ColorChoice, Command};

mod cmd_necom;

fn main() -> anyhow::Result<()> {
    // Default to `info` level so progress/warning messages remain visible by default,
    // matching the previous `eprintln!` behavior. Users can override via RUST_LOG.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    let app = Command::new("necom")
        .version(crate_version!())
        .author(crate_authors!())
        .about("`necom` - Clustering, Matrix, and Phylogeny Toolkit")
        .propagate_version(true)
        .arg_required_else_help(true)
        .color(ColorChoice::Auto)
        .subcommand(cmd_necom::clust::make_subcommand())
        .subcommand(cmd_necom::cut::make_subcommand())
        .subcommand(cmd_necom::eval::make_subcommand())
        .subcommand(cmd_necom::mat::make_subcommand())
        .subcommand(cmd_necom::nwk::make_subcommand())
        .subcommand(cmd_necom::pl::make_subcommand())
        .after_help(
            r###"Subcommand groups:

* Clustering:
    * clust - Algorithms: cc, dbscan, hier, k-medoids, mcl, nj, upgma

* Tree cutting:
    * cut   - Cut a Newick tree into flat partitions

* Evaluation:
    * eval - Metrics: compare, partition, replicate

* Matrix:
    * mat   - Processing: compare, format, subset, to-pair, to-phylip, transform

* Phylogeny:
    * nwk   - Newick tools: stat, distance, reroot, prune, label, order, indent, comment, rename, replace, subtree, topo, to-dot, to-forest, to-svg, to-tex

* Pipelines:
    * pl - Workflows: condense

"###,
        );

    match app.get_matches().subcommand() {
        Some(("clust", sub_matches)) => cmd_necom::clust::execute(sub_matches),
        Some(("cut", sub_matches)) => cmd_necom::cut::execute(sub_matches),
        Some(("eval", sub_matches)) => cmd_necom::eval::execute(sub_matches),
        Some(("mat", sub_matches)) => cmd_necom::mat::execute(sub_matches),
        Some(("nwk", sub_matches)) => cmd_necom::nwk::execute(sub_matches),
        Some(("pl", sub_matches)) => cmd_necom::pl::execute(sub_matches),
        _ => anyhow::bail!("unknown subcommand"),
    }?;

    Ok(())
}
