use anyhow::Result;
use clap::{ArgMatches, Command};
use necom::libs::tree_cut as cut;

use crate::cmd_necom::args;
use crate::cmd_necom::cut::{init_stats_writer, load_tree};

/// Build the `cut scan-dynamic` subcommand.
pub fn make_subcommand() -> Command {
    Command::new("scan-dynamic")
        .about("Scans dynamic tree cut min cluster sizes")
        .after_help(include_str!("../../../docs/help/cut/scan-dynamic.md"))
        .arg(args::infile_arg_required_with_help("Input Newick file"))
        .arg(args::range_arg())
        .arg(args::deep_split_arg())
        .arg(args::max_tree_height_arg())
        .arg(args::stats_out_arg())
        .arg(args::support_arg())
}

/// Execute the `cut scan-dynamic` subcommand.
pub fn execute(args: &ArgMatches) -> Result<()> {
    let tree = load_tree(args)?;

    let range_str = args.get_one::<String>("range").unwrap();
    let (start, end, step) = parse_integer_range(range_str)?;

    let deep_split = args.get_flag("deep_split");
    let max_tree_height = args.get_one::<f64>("max_tree_height").copied();

    let params = cut::scan::ScanParams {
        start,
        end,
        step,
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

/// Parse a scan range and validate that all three values are non-negative integers.
fn parse_integer_range(range_str: &str) -> Result<(f64, f64, f64)> {
    let parts: Vec<&str> = range_str.split(',').collect();
    if parts.len() != 3 {
        anyhow::bail!("--range format must be start,end,step");
    }

    let labels = ["start", "end", "step"];
    let mut values = [0usize; 3];
    for (i, part) in parts.iter().enumerate() {
        match part.parse::<usize>() {
            Ok(v) => values[i] = v,
            Err(_) => {
                anyhow::bail!(
                    "--range {} must be a non-negative integer, got {}",
                    labels[i],
                    part
                );
            }
        }
    }

    let start = values[0] as f64;
    let end = values[1] as f64;
    let step = values[2] as f64;

    if step <= 0.0 {
        anyhow::bail!("--range step must be positive");
    }

    Ok((start, end, step))
}
