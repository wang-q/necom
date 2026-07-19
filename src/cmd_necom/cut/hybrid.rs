use anyhow::{Context, Result};
use clap::{ArgMatches, Command};
use necom::libs::tree_cut as cut;

use crate::cmd_necom::args;
use crate::cmd_necom::cut::{load_tree, write_clusters, DispatchBuilder, OutputOptions};

/// Build the `cut hybrid` subcommand.
pub fn make_subcommand() -> Command {
    Command::new("hybrid")
        .about("Cuts a tree using dynamic hybrid cut")
        .after_help(include_str!("../../../docs/help/cut/hybrid.md"))
        .arg(args::infile_arg_required_with_help("Input Newick file"))
        .arg(args::matrix_arg().required(true))
        .arg(args::min_size_arg())
        .arg(args::max_pam_dist_arg())
        .arg(args::no_pam_dendro_arg())
        .arg(args::deep_split_arg())
        .arg(args::max_tree_height_arg())
        .arg(args::format_arg())
        .arg(args::rep_arg())
        .arg(args::outfile_arg())
        .arg(args::support_arg())
}

/// Execute the `cut hybrid` subcommand.
pub fn execute(args: &ArgMatches) -> Result<()> {
    let tree = load_tree(args)?;

    let min_size = *args
        .get_one::<usize>("min_size")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: --min-size"))?;

    let matrix_file = args.get_one::<String>("matrix").unwrap();
    let matrix = necom::libs::pairmat::NamedMatrix::from_relaxed_phylip(matrix_file)
        .with_context(|| format!("Failed to load matrix from {}", matrix_file))?;

    let max_pam_dist = args.get_one::<f64>("max_pam_dist").copied();
    let no_pam_dendro = args.get_flag("no_pam_dendro");
    let deep_split = args.get_flag("deep_split");
    let max_tree_height = args.get_one::<f64>("max_tree_height").copied();

    let dispatch = DispatchBuilder::hybrid(min_size, matrix)
        .max_pam_dist(max_pam_dist)
        .no_pam_dendro(no_pam_dendro)
        .deep_split(deep_split)
        .max_tree_height(max_tree_height)
        .build(&tree)?;
    let (partition, _) = cut::dispatch_cut(&tree, dispatch)?;

    let opts = OutputOptions::from_args(args)?;
    let clusters = cut::partition_to_clusters(&partition, &tree, opts.rep_mode);
    write_clusters(&clusters, &opts)?;

    Ok(())
}
