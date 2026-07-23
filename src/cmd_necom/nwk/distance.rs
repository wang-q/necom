use anyhow::{anyhow, Context};
use clap::{ArgMatches, Command};
use necom::libs::phylo::tree::{distance, Tree};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

/// Build the clap subcommand for distance.
pub fn make_subcommand() -> Command {
    Command::new("distance")
        .about("Calculates distances between nodes")
        .after_help(include_str!("../../../docs/help/nwk/distance.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(
            crate::cmd_necom::args::mode_arg(
                "root",
                &["root", "parent", "pairwise", "lca", "phylip"],
                "Set the mode for calculating distances",
            )
            .short('m'),
        )
        .arg(crate::cmd_necom::args::internal_arg())
        .arg(crate::cmd_necom::args::leaf_arg())
        .arg(crate::cmd_necom::args::node_arg())
        .arg(crate::cmd_necom::args::name_list_arg())
        .arg(crate::cmd_necom::args::regex_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the distance command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let trees = Tree::from_file(infile)?;

    if trees.len() > 1 {
        log::warn!(
            "file contains {} trees, only the first will be processed",
            trees.len()
        );
    }

    let tree = trees
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no trees found in {}", infile))?;

    let mode = args
        .get_one::<String>("mode")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: mode"))?;

    // Select nodes, applying -I/-L filters consistently with other commands.
    // When no name-based filter is given, include all selected nodes (not only
    // named ones) so that internal/unnamed nodes can be reported too.
    let ids_pos = super::common::match_positions(&tree, args)?;
    let ids_name = super::common::match_names(&tree, args, true)?;
    let has_name_filter = args.try_contains_id("node").unwrap_or(false)
        || args.try_contains_id("name_list").unwrap_or(false)
        || args.try_contains_id("regex").unwrap_or(false);

    let ids: BTreeSet<usize> = if has_name_filter {
        ids_pos.intersection(&ids_name).cloned().collect()
    } else {
        ids_pos
    };

    if mode == "phylip" {
        for &id in &ids {
            if tree.get_node(id).map(|n| n.name.is_none()).unwrap_or(false) {
                anyhow::bail!("Phylip matrix requires all selected nodes to be named");
            }
            if let Some(name) = tree.get_node(id).and_then(|n| n.name.as_ref()) {
                if name.contains(|c: char| c.is_ascii_whitespace()) {
                    anyhow::bail!(
                        "Phylip matrix requires node names without whitespace: '{}'",
                        name
                    );
                }
            }
        }
    }

    let mut id_of = BTreeMap::new();
    let name_id_map = tree.get_name_id();
    let existing_names: std::collections::HashSet<&String> =
        name_id_map.keys().collect();
    // Build a reverse id -> name map for O(1) lookup per selected id.
    let id_name_map: std::collections::HashMap<usize, &String> =
        name_id_map.iter().map(|(n, &i)| (i, n)).collect();
    for &id in &ids {
        let label = if let Some(&name) = id_name_map.get(&id) {
            name.clone()
        } else {
            // Synthetic label for unnamed nodes; ensure it does not collide.
            let mut synthetic = format!("#{}", id);
            let mut suffix = 1usize;
            while existing_names.contains(&synthetic) {
                synthetic = format!("#{}_{}", id, suffix);
                suffix += 1;
            }
            synthetic
        };
        id_of.insert(label, id);
    }

    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    match mode.as_str() {
        "root" => distance::dist_root(&tree, &id_of, &mut writer)?,
        "parent" => distance::dist_parent(&tree, &id_of, &mut writer)?,
        "pairwise" => distance::dist_pairwise(&tree, &id_of, &mut writer)?,
        "lca" => distance::dist_lca(&tree, &id_of, &mut writer)?,
        "phylip" => distance::dist_phylip(&tree, &id_of, &mut writer)?,
        _ => anyhow::bail!("unknown distance mode: {}", mode),
    }

    writer.flush()?;
    Ok(())
}
