use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use necom::libs::tree_cut as cut;

use crate::cmd_necom::args;
use crate::cmd_necom::cut::{init_stats_writer, load_tree, normalize_method_name};

/// Build the `cut scan-simple` subcommand.
pub fn make_subcommand() -> Command {
    Command::new("scan-simple")
        .about("Scans static threshold cut parameters")
        .after_help(include_str!("../../../docs/help/cut/scan-simple.md"))
        .arg(args::infile_arg_required_with_help("Input Newick file"))
        .arg(
            Arg::new("method")
                .long("method")
                .required(true)
                .num_args(1)
                .value_parser([
                    "k",
                    "height",
                    "root-dist",
                    "max-clade",
                    "avg-clade",
                    "med-clade",
                    "sum-branch",
                    "leaf-dist-max",
                    "leaf-dist-min",
                    "leaf-dist-avg",
                    "max-edge",
                    "inconsistent",
                ])
                .help("Cutting method to sweep"),
        )
        .arg(args::range_arg())
        .arg(args::deep_arg())
        .arg(args::stats_out_arg())
        .arg(args::support_arg())
}

/// Execute the `cut scan-simple` subcommand.
pub fn execute(args: &ArgMatches) -> Result<()> {
    let tree = load_tree(args)?;

    let method_name = args.get_one::<String>("method").unwrap();
    let name = normalize_method_name(method_name)?;

    let range_str = args.get_one::<String>("range").unwrap();
    let (start, end, step) = cut::scan::parse_scan_range(range_str)?;

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
