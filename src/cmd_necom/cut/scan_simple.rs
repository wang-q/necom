use anyhow::Result;
use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command};
use necom::libs::tree_cut as cut;

use crate::cmd_necom::args;
use crate::cmd_necom::cut::{init_stats_writer, load_tree};

/// Build the `cut scan-simple` subcommand.
pub fn make_subcommand() -> Command {
    let mut cmd = Command::new("scan-simple")
        .about("Scans static threshold cut parameters")
        .after_help(include_str!("../../../docs/help/cut/scan-simple.md"))
        .arg(args::infile_arg_required_with_help("Input Newick file"))
        .arg(
            Arg::new("k")
                .long("k")
                .action(ArgAction::SetTrue)
                .help("Sweep over cluster count K"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .action(ArgAction::SetTrue)
                .help("Sweep over height threshold"),
        )
        .arg(
            Arg::new("root_dist")
                .long("root-dist")
                .action(ArgAction::SetTrue)
                .help("Sweep over root-distance threshold"),
        )
        .arg(
            Arg::new("max_clade")
                .long("max-clade")
                .action(ArgAction::SetTrue)
                .help("Sweep over max pairwise distance threshold"),
        )
        .arg(
            Arg::new("avg_clade")
                .long("avg-clade")
                .action(ArgAction::SetTrue)
                .help("Sweep over average pairwise distance threshold"),
        )
        .arg(
            Arg::new("med_clade")
                .long("med-clade")
                .action(ArgAction::SetTrue)
                .help("Sweep over median pairwise distance threshold"),
        )
        .arg(
            Arg::new("sum_branch")
                .long("sum-branch")
                .action(ArgAction::SetTrue)
                .help("Sweep over sum-of-branch-length threshold"),
        )
        .arg(
            Arg::new("leaf_dist_max")
                .long("leaf-dist-max")
                .action(ArgAction::SetTrue)
                .help("Sweep over max leaf-distance threshold"),
        )
        .arg(
            Arg::new("leaf_dist_min")
                .long("leaf-dist-min")
                .action(ArgAction::SetTrue)
                .help("Sweep over min leaf-distance threshold"),
        )
        .arg(
            Arg::new("leaf_dist_avg")
                .long("leaf-dist-avg")
                .action(ArgAction::SetTrue)
                .help("Sweep over average leaf-distance threshold"),
        )
        .arg(
            Arg::new("max_edge")
                .long("max-edge")
                .alias("single-linkage")
                .action(ArgAction::SetTrue)
                .help("Sweep over max edge-length threshold"),
        )
        .arg(
            Arg::new("inconsistent")
                .long("inconsistent")
                .action(ArgAction::SetTrue)
                .help("Sweep over inconsistency coefficient threshold"),
        )
        .arg(args::range_arg().required(true))
        .arg(args::deep_arg())
        .arg(args::stats_out_arg())
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

/// Execute the `cut scan-simple` subcommand.
pub fn execute(args: &ArgMatches) -> Result<()> {
    let tree = load_tree(args)?;

    let name = detect_method(args)?;
    let range_str = args.get_one::<String>("range").unwrap();

    let (start, end, step) = if name == "k" {
        let (s, e, st) = cut::scan::parse_scan_range_usize(range_str)?;
        if s < 1 {
            anyhow::bail!("--range start must be at least 1 for cluster count scanning");
        }
        (s as f64, e as f64, st as f64)
    } else {
        cut::scan::parse_scan_range(range_str)?
    };

    let deep = *args.get_one::<usize>("deep").unwrap();

    let params = cut::scan::ScanParams {
        start,
        end,
        step,
        method_name: Some(name),
        dynamic_tree: false,
    };

    let mut writer = necom::writer("stdout")?;
    let mut stats_writer = init_stats_writer(args)?;

    cut::scan::run_scan(
        &tree,
        &mut writer,
        &mut stats_writer,
        params,
        deep,
        None,
        false,
        false,
        None,
    )
}

/// Detect which static cut method flag was requested and return its normalized name.
fn detect_method(args: &ArgMatches) -> Result<&'static str> {
    for &name in cut::METHOD_NAMES {
        if args.get_flag(name) {
            return Ok(name);
        }
    }
    anyhow::bail!("no cut method specified")
}
