use anyhow::Context;
use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command, Id};
use necom::libs::phylo::tree::{algo, Tree};
use std::collections::HashSet;
use std::io::Write;

/// Build the clap subcommand for order.
pub fn make_subcommand() -> Command {
    Command::new("order")
        .about("Orders nodes in a Newick file")
        .after_help(include_str!("../../../docs/help/nwk/order.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(
            Arg::new("num_descendants")
                .long("num-descendants")
                .action(ArgAction::SetTrue)
                .help("By number of descendants"),
        )
        .arg(
            Arg::new("num_descendants_rev")
                .long("num-descendants-rev")
                .action(ArgAction::SetTrue)
                .help("By number of descendants, reversely"),
        )
        .group(
            ArgGroup::new("number-of-descendants")
                .args(["num_descendants", "num_descendants_rev"]),
        )
        .arg(
            Arg::new("alphanumeric")
                .long("alphanumeric")
                .action(ArgAction::SetTrue)
                .help("By alphanumeric order of labels"),
        )
        .arg(
            Arg::new("alphanumeric_rev")
                .long("alphanumeric-rev")
                .action(ArgAction::SetTrue)
                .help("By alphanumeric order of labels, reversely"),
        )
        .group(
            ArgGroup::new("alphanumeric-order")
                .args(["alphanumeric", "alphanumeric_rev"]),
        )
        .arg(
            Arg::new("deladderize")
                .long("deladderize")
                .visible_alias("dl")
                .action(ArgAction::SetTrue)
                .help("De-ladderize (alternate) the tree"),
        )
        .arg(crate::cmd_necom::args::name_list_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the order command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    let opt_nd = match args.get_one::<Id>("number-of-descendants") {
        None => "",
        Some(x) => x.as_str(),
    };
    let opt_an = match args.get_one::<Id>("alphanumeric-order") {
        None => "",
        Some(x) => x.as_str(),
    };

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let mut trees = Tree::from_file(infile)?;

    let mut names = vec![];
    if args.contains_id("name_list") {
        let list_file = args
            .get_one::<String>("name_list")
            .ok_or_else(|| anyhow::anyhow!("missing required argument: name_list"))?;
        names = necom::libs::io::read_names::<Vec<String>>(list_file)?;
    }

    let is_deladderize = args.get_flag("deladderize");

    // Default behavior: if no specific sort order is requested, use alphanumeric
    let default_an =
        names.is_empty() && opt_an.is_empty() && opt_nd.is_empty() && !is_deladderize;

    for tree in &mut trees {
        if !names.is_empty() {
            let leaf_name_vec = tree.get_leaf_names();
            let leaf_names: HashSet<&str> =
                leaf_name_vec.iter().filter_map(|n| n.as_deref()).collect();
            let missing: Vec<&str> = names
                .iter()
                .map(|s| s.as_str())
                .filter(|n| !leaf_names.contains(n))
                .collect();
            if !missing.is_empty() {
                log::warn!("name-list entries not found in tree: {:?}", missing);
            }
            algo::sort_by_list(tree, &names);
        }
        if default_an || !opt_an.is_empty() {
            algo::sort_by_name(tree, opt_an == "alphanumeric_rev");
        }
        if !opt_nd.is_empty() {
            algo::ladderize(tree, opt_nd == "num_descendants_rev");
        }
        if is_deladderize {
            algo::deladderize(tree);
        }

        let out_string = tree.to_newick();
        writer.write_fmt(format_args!("{}\n", out_string))?;
    }

    writer.flush()?;
    Ok(())
}
