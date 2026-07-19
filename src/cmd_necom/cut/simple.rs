use anyhow::Result;
use clap::{value_parser, Arg, ArgMatches, Command};
use necom::libs::tree_cut as cut;

use crate::cmd_necom::args;
use crate::cmd_necom::cut::{load_tree, write_clusters, DispatchBuilder, OutputOptions};

/// Build the `cut simple` subcommand.
pub fn make_subcommand() -> Command {
    Command::new("simple")
        .about("Cuts a tree using a static threshold method")
        .after_help(include_str!("../../../docs/help/cut/simple.md"))
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
                .help("Cutting method"),
        )
        .arg(
            Arg::new("threshold")
                .long("threshold")
                .required(true)
                .num_args(1)
                .value_parser(value_parser!(f64))
                .help("Threshold value for the method"),
        )
        .arg(args::deep_arg())
        .arg(args::format_arg())
        .arg(args::rep_arg())
        .arg(args::outfile_arg())
        .arg(args::support_arg())
}

/// Execute the `cut simple` subcommand.
pub fn execute(args: &ArgMatches) -> Result<()> {
    let tree = load_tree(args)?;

    let method_name = args.get_one::<String>("method").unwrap();
    let name = crate::cmd_necom::cut::normalize_method_name(method_name)?;

    let threshold = *args.get_one::<f64>("threshold").unwrap();
    let val = if name == "k" {
        if threshold < 1.0 || threshold.fract() != 0.0 {
            anyhow::bail!("k must be a positive integer, got {}", threshold);
        }
        threshold
    } else {
        threshold
    };

    let deep = *args.get_one::<usize>("deep").unwrap();

    let dispatch = DispatchBuilder::standard(name, val, deep).build(&tree)?;
    let (partition, _) = cut::dispatch_cut(&tree, dispatch)?;

    let opts = OutputOptions::from_args(args)?;
    let clusters = cut::partition_to_clusters(&partition, &tree, opts.rep_mode);
    write_clusters(&clusters, &opts)?;

    Ok(())
}
