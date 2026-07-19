use anyhow::Result;
use clap::{value_parser, Arg, ArgGroup, ArgMatches, Command};
use necom::libs::cut;

use crate::cmd_necom::args;
use crate::cmd_necom::cut::{load_tree, write_clusters, DispatchBuilder, OutputOptions};

/// Build the `cut simple` subcommand.
pub fn make_subcommand() -> Command {
    let mut cmd = Command::new("simple")
        .about("Cuts a tree using a static threshold method")
        .after_help(include_str!("../../../docs/help/cut/simple.md"))
        .arg(args::infile_arg_required_with_help("Input Newick file"))
        .arg(args::k_arg())
        .arg(
            Arg::new("height")
                .long("height")
                .value_parser(value_parser!(f64))
                .help("Cut at specific height (max distance to leaves)"),
        )
        .arg(
            Arg::new("root_dist")
                .long("root-dist")
                .value_parser(value_parser!(f64))
                .help("Cut at specific distance from root"),
        )
        .arg(
            Arg::new("max_clade")
                .long("max-clade")
                .value_parser(value_parser!(f64))
                .help("Max pairwise distance in cluster threshold"),
        )
        .arg(
            Arg::new("avg_clade")
                .long("avg-clade")
                .value_parser(value_parser!(f64))
                .help("Average pairwise distance in cluster threshold"),
        )
        .arg(
            Arg::new("med_clade")
                .long("med-clade")
                .value_parser(value_parser!(f64))
                .help("Median pairwise distance in cluster threshold"),
        )
        .arg(
            Arg::new("sum_branch")
                .long("sum-branch")
                .value_parser(value_parser!(f64))
                .help("Sum of branch lengths in cluster threshold"),
        )
        .arg(
            Arg::new("leaf_dist_max")
                .long("leaf-dist-max")
                .value_parser(value_parser!(f64))
                .help("Max distance from cluster root to any leaf"),
        )
        .arg(
            Arg::new("leaf_dist_min")
                .long("leaf-dist-min")
                .value_parser(value_parser!(f64))
                .help("Min distance from cluster root to any leaf"),
        )
        .arg(
            Arg::new("leaf_dist_avg")
                .long("leaf-dist-avg")
                .value_parser(value_parser!(f64))
                .help("Average distance from cluster root to leaves"),
        )
        .arg(
            Arg::new("max_edge")
                .long("max-edge")
                .alias("single-linkage")
                .value_parser(value_parser!(f64))
                .help("Cut branches longer than threshold (Single Linkage)"),
        )
        .arg(
            Arg::new("inconsistent")
                .long("inconsistent")
                .value_parser(value_parser!(f64))
                .help("Cut by inconsistent coefficient threshold"),
        )
        .arg(args::deep_arg())
        .arg(args::format_arg())
        .arg(args::rep_arg())
        .arg(args::outfile_arg())
        .arg(args::support_arg());

    cmd = cmd.group(
        ArgGroup::new("method")
            .args([
                "k",
                "height",
                "root_dist",
                "max_clade",
                "avg_clade",
                "med_clade",
                "sum_branch",
                "leaf_dist_max",
                "leaf_dist_min",
                "leaf_dist_avg",
                "max_edge",
                "inconsistent",
            ])
            .required(true),
    );
    cmd
}

/// Execute the `cut simple` subcommand.
pub fn execute(args: &ArgMatches) -> Result<()> {
    let tree = load_tree(args)?;

    let (name, val) = detect_method_and_threshold(args)?;
    let deep = *args.get_one::<usize>("deep").unwrap();

    let dispatch = DispatchBuilder::standard(name, val, deep).build(&tree)?;
    let (partition, _) = cut::dispatch_cut(&tree, dispatch)?;

    let opts = OutputOptions::from_args(args)?;
    let clusters = cut::partition_to_clusters(&partition, &tree, opts.rep_mode);
    write_clusters(&clusters, &opts)?;

    Ok(())
}

/// Detect which static cut method was requested and return its normalized name
/// and threshold value.
fn detect_method_and_threshold(args: &ArgMatches) -> Result<(&'static str, f64)> {
    if let Some(&k) = args.get_one::<usize>("k") {
        if k < 1 {
            anyhow::bail!("k must be a positive integer, got {}", k);
        }
        return Ok(("k", k as f64));
    }

    for &name in &[
        "height",
        "root_dist",
        "max_clade",
        "avg_clade",
        "med_clade",
        "sum_branch",
        "leaf_dist_max",
        "leaf_dist_min",
        "leaf_dist_avg",
        "max_edge",
        "inconsistent",
    ] {
        if let Some(&val) = args.get_one::<f64>(name) {
            return Ok((name, val));
        }
    }

    anyhow::bail!("no cut method specified")
}
