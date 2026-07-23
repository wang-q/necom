use anyhow::Context;
use clap::{builder, Arg, ArgAction, ArgGroup, ArgMatches, Command, Id};
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
                .visible_alias("nd")
                .action(ArgAction::SetTrue)
                .help("By number of descendants"),
        )
        .arg(
            Arg::new("num_descendants_rev")
                .long("num-descendants-rev")
                .visible_alias("ndr")
                .action(ArgAction::SetTrue)
                .help("By number of descendants, in reverse order"),
        )
        .group(
            ArgGroup::new("number-of-descendants")
                .args(["num_descendants", "num_descendants_rev"]),
        )
        .arg(
            Arg::new("alphanumeric")
                .long("alphanumeric")
                .visible_alias("an")
                .action(ArgAction::SetTrue)
                .help("By alphanumeric order of labels"),
        )
        .arg(
            Arg::new("alphanumeric_rev")
                .long("alphanumeric-rev")
                .visible_alias("anr")
                .action(ArgAction::SetTrue)
                .help("By alphanumeric order of labels, in reverse order"),
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
        .arg(
            Arg::new("olo")
                .long("olo")
                .num_args(1)
                .conflicts_with("number-of-descendants")
                .conflicts_with("alphanumeric-order")
                .conflicts_with("name_list")
                .conflicts_with("deladderize")
                .help("Optimal leaf ordering using a distance matrix"),
        )
        .arg(
            Arg::new("olo_format")
                .long("olo-format")
                .default_value("phylip")
                .value_parser([
                    builder::PossibleValue::new("phylip"),
                    builder::PossibleValue::new("pair"),
                ])
                .help("Format of the --olo distance matrix"),
        )
        .arg(crate::cmd_necom::args::same_arg("0.0"))
        .arg(crate::cmd_necom::args::missing_arg("1.0"))
        .arg(crate::cmd_necom::args::name_list_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the order command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
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
    let olo_file = args.get_one::<String>("olo");

    // Default behavior: if no specific sort order is requested, use alphanumeric
    let default_an = names.is_empty()
        && opt_an.is_empty()
        && opt_nd.is_empty()
        && !is_deladderize
        && olo_file.is_none();

    let olo_matrix = if let Some(path) = olo_file {
        let format = args
            .get_one::<String>("olo_format")
            .map(|s| s.as_str())
            .unwrap_or("phylip");
        if format == "pair" {
            let same = *args
                .get_one::<f32>("same")
                .context("missing required argument: same")?;
            let missing = *args
                .get_one::<f32>("missing")
                .context("missing required argument: missing")?;
            Some(necom::libs::pairmat::NamedMatrix::from_pair_scores(
                path, same, missing,
            )?)
        } else {
            Some(necom::libs::pairmat::NamedMatrix::from_relaxed_phylip(
                path,
            )?)
        }
    } else {
        None
    };

    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    for tree in &mut trees {
        if let Some(ref matrix) = olo_matrix {
            algo::optimal_leaf_order(tree, matrix)?;
        } else {
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
                    anyhow::bail!(
                        "name-list entries not found in tree: {}",
                        missing.join(", ")
                    );
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
        }

        let out_string = tree.to_newick();
        writer.write_fmt(format_args!("{}\n", out_string))?;
    }

    writer.flush()?;
    Ok(())
}
