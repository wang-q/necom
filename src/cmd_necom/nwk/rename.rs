use anyhow::Context;
use clap::{Arg, ArgAction, ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
use std::collections::BTreeMap;
use std::io::Write;

/// Build the clap subcommand for rename.
pub fn make_subcommand() -> Command {
    Command::new("rename")
        .about("Renames nodes in a Newick file")
        .after_help(include_str!("../../../docs/help/nwk/rename.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::node_arg())
        .arg(crate::cmd_necom::args::lca_arg())
        .arg(
            Arg::new("rename")
                .long("rename")
                .num_args(1)
                .required(true)
                .action(ArgAction::Append)
                .help("New name"),
        )
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the rename command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut names = vec![];
    if args.contains_id("node") {
        for name in args
            .get_many::<String>("node")
            .ok_or_else(|| anyhow::anyhow!("missing required argument: node"))?
        {
            names.push(name.to_string());
        }
    }

    let mut lcas = vec![];
    if args.contains_id("lca") {
        for lca in args
            .get_many::<String>("lca")
            .ok_or_else(|| anyhow::anyhow!("missing required argument: lca"))?
        {
            lcas.push(lca.to_string());
        }
    }

    let mut renames = vec![];
    for rename in args
        .get_many::<String>("rename")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: rename"))?
    {
        renames.push(rename.to_string());
    }

    // The sum of --node and --lca must equal the number of --rename
    anyhow::ensure!(
        names.len() + lcas.len() == renames.len(),
        "the number of --node ({}) plus --lca ({}) must equal the number of --rename ({})",
        names.len(),
        lcas.len(),
        renames.len()
    );
    let len_names = names.len();

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let mut trees = Tree::from_file(infile)?;

    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    for tree in &mut trees {
        // ids with names
        let id_of: BTreeMap<_, _> = tree.get_name_id();
        let duplicates = necom::libs::phylo::tree::stat::duplicate_names(tree);

        // all IDs to be modified
        let mut rename_of: BTreeMap<_, _> = BTreeMap::new();

        // ids supplied by --node
        for (i, name) in names.iter().enumerate() {
            if let Some(id) = id_of.get(name) {
                super::common::warn_duplicate_name(&duplicates, name);
                let rename = renames.get(i).ok_or_else(|| {
                    anyhow::anyhow!("rename entry missing at index {}", i)
                })?;
                rename_of.insert(*id, rename.to_string());
            } else {
                log::warn!("node not found: {}", name);
            }
        }

        // ids supplied by --lca
        for (i, lca) in lcas.iter().enumerate() {
            let (first, last) = super::common::parse_lca_pair(lca)?;
            super::common::warn_duplicate_name(&duplicates, first);
            super::common::warn_duplicate_name(&duplicates, last);

            match (id_of.get(first), id_of.get(last)) {
                (Some(id1), Some(id2)) => {
                    let x = tree.get_common_ancestor(*id1, *id2)?;
                    let rename = renames.get(len_names + i).ok_or_else(|| {
                        anyhow::anyhow!(
                            "rename entry missing at index {}",
                            len_names + i
                        )
                    })?;
                    rename_of.insert(x, rename.to_string());
                }
                _ => {
                    log::warn!("lca name not found in tree: {} / {}", first, last);
                }
            }
        }

        for (k, v) in &rename_of {
            if let Some(node) = tree.get_node_mut(*k) {
                node.name = Some(v.to_string());
            }
        }

        let out_string = tree.to_newick();
        writer.write_fmt(format_args!("{}\n", out_string))?;
    }

    writer.flush()?;
    Ok(())
}
