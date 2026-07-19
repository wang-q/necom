use anyhow::Result;
use clap::{ArgMatches, Command};
use necom::libs::cut;

use crate::cmd_necom::args;
use crate::cmd_necom::cut::{init_stats_writer, load_tree};

/// Build the `cut scan-dynamic` subcommand.
pub fn make_subcommand() -> Command {
    Command::new("scan-dynamic")
        .about("Scans dynamic tree cut min cluster sizes")
        .after_help(include_str!("../../../docs/help/cut/scan-dynamic.md"))
        .arg(args::infile_arg_required_with_help("Input Newick file"))
        .arg(args::range_arg().required(true))
        .arg(args::deep_split_arg())
        .arg(args::max_tree_height_arg())
        .arg(args::stats_out_arg())
        .arg(args::support_arg())
}

/// Execute the `cut scan-dynamic` subcommand.
pub fn execute(args: &ArgMatches) -> Result<()> {
    let tree = load_tree(args)?;

    let range_str = args.get_one::<String>("range").unwrap();
    let (start, end, step) = cut::scan::parse_scan_range_usize(range_str)?;

    let deep_split = args.get_flag("deep_split");
    let max_tree_height = args.get_one::<f64>("max_tree_height").copied();

    let params = cut::scan::ScanParams {
        start: start as f64,
        end: end as f64,
        step: step as f64,
        method_name: None,
        dynamic_tree: true,
    };

    let mut writer = necom::writer("stdout")?;
    let mut stats_writer = init_stats_writer(args)?;

    cut::scan::run_scan(
        &tree,
        &mut writer,
        &mut stats_writer,
        params,
        2,
        max_tree_height,
        deep_split,
        false,
        None,
    )
}
