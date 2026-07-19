use anyhow::Result;
use clap::{ArgMatches, Command};
use necom::libs::cut;

use crate::cmd_necom::args;
use crate::cmd_necom::cut::{load_tree, write_clusters, DispatchBuilder, OutputOptions};

/// Build the `cut dynamic` subcommand.
pub fn make_subcommand() -> Command {
    Command::new("dynamic")
        .about("Cuts a tree using dynamic tree cut")
        .after_help(include_str!("../../../docs/help/cut/dynamic.md"))
        .arg(args::infile_arg_required_with_help("Input Newick file"))
        .arg(args::min_size_arg().required(true))
        .arg(args::deep_split_arg())
        .arg(args::max_tree_height_arg())
        .arg(args::format_arg())
        .arg(args::rep_arg())
        .arg(args::outfile_arg())
        .arg(args::support_arg())
}

/// Execute the `cut dynamic` subcommand.
pub fn execute(args: &ArgMatches) -> Result<()> {
    let tree = load_tree(args)?;

    let min_size = *args.get_one::<usize>("min_size").unwrap();

    let deep_split = args.get_flag("deep_split");
    let max_tree_height = args.get_one::<f64>("max_tree_height").copied();

    let dispatch = DispatchBuilder::dynamic(min_size)
        .deep_split(deep_split)
        .max_tree_height(max_tree_height)
        .build(&tree)?;
    let (partition, _) = cut::dispatch_cut(&tree, dispatch)?;

    let opts = OutputOptions::from_args(args)?;
    let clusters = cut::partition_to_clusters(&partition, &tree, opts.rep_mode);
    write_clusters(&clusters, &opts)?;

    Ok(())
}
