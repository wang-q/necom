use clap::{crate_authors, crate_version, ColorChoice, Command};

mod cmd_pgr;

fn main() -> anyhow::Result<()> {
    // Default to `info` level so progress/warning messages remain visible by default,
    // matching the previous `eprintln!` behavior. Users can override via RUST_LOG.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let app = Command::new("pgr")
        .version(crate_version!())
        .author(crate_authors!())
        .about("`pgr` - Practical Genome Refiner")
        .propagate_version(true)
        .arg_required_else_help(true)
        .color(ColorChoice::Auto)
        .subcommand(cmd_pgr::clust::make_subcommand())
        .subcommand(cmd_pgr::dist::make_subcommand())
        .subcommand(cmd_pgr::mat::make_subcommand())
        .subcommand(cmd_pgr::nwk::make_subcommand())
        .subcommand(cmd_pgr::pl::make_subcommand())
        .after_help(
            r###"Subcommand groups:

* Clustering:
    * clust - Algorithms: cc, cut, dbscan, eval, hier, k-medoids, mcl, nj, upgma

* Distance:
    * dist  - Metrics: hv, seq, vector

* Matrix:
    * mat   - Processing: compare, format, subset, to-pair, to-phylip, transform

* Phylogeny:
    * nwk   - Newick tools: stat, distance, cmp, reroot, prune, label, order, indent, comment, rename, replace, subtree, support, topo, to-dot, to-forest, to-svg, to-tex

* Pipelines:
    * pl - Workflows: condense, p2m, prefilter, trf, ir, rept, ucsc

"###,
        );

    // Check which subcommand the user ran...
    match app.get_matches().subcommand() {
        Some(("clust", sub_matches)) => cmd_pgr::clust::execute(sub_matches),
        Some(("dist", sub_matches)) => cmd_pgr::dist::execute(sub_matches),
        Some(("mat", sub_matches)) => cmd_pgr::mat::execute(sub_matches),
        Some(("nwk", sub_matches)) => cmd_pgr::nwk::execute(sub_matches),
        Some(("pl", sub_matches)) => cmd_pgr::pl::execute(sub_matches),
        _ => anyhow::bail!("unknown subcommand"),
    }?;

    Ok(())
}
