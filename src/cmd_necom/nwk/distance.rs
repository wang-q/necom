use anyhow::{anyhow, Context};
use clap::{ArgMatches, Command};
use necom::libs::phylo::tree::{distance, Tree};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

/// Build the clap subcommand for distance.
pub fn make_subcommand() -> Command {
    Command::new("distance")
        .about("Calculates distances between nodes")
        .after_help(
            r###"
Calculates distances between nodes or generates distance matrices.

Notes:
* Modes:
    * `root`: Distance from each node to the root.
      Output: Node \t Distance
    * `parent`: Distance from each node to its parent.
      Output: Node \t Distance
    * `pairwise`: Distance between every pair of selected nodes, including self-pairs and both (i,j) and (j,i) orderings.
      Output: Node1 \t Node2 \t Distance
    * `lca`: Distance from each node in a pair to their Lowest Common Ancestor (LCA), for all selected-node pairs (including self-pairs).
      Output: Node1 \t Node2 \t Dist1 \t Dist2
    * `phylip`: A Phylip-formatted distance matrix for the selected nodes.

* The `-I` and `-L` options filter out internal or leaf nodes.
* Use `-n` / `-l` / `-x` to restrict the reported nodes to a name, name-list file, or regex.
* When no name-based filter is given, all selected nodes (respecting `-I`/`-L`) are reported.
* Input must be a valid Newick file.

Examples:
1. Distances to root (default):
   necom nwk distance tree.nwk

2. Pairwise distances:
   necom nwk distance tree.nwk --mode pairwise

3. Generate Phylip matrix:
   necom nwk distance tree.nwk --mode phylip > matrix.phy

4. Distances to parent for leaves only:
   necom nwk distance tree.nwk --mode parent -I

5. Distance to root for selected nodes only:
   necom nwk distance tree.nwk --mode root -n Homo -n Pan
"###,
        )
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::mode_arg(
            "root",
            &["root", "parent", "pairwise", "lca", "phylip"],
            "Set the mode for calculating distances",
        ))
        .arg(crate::cmd_necom::args::internal_arg())
        .arg(crate::cmd_necom::args::leaf_arg())
        .arg(crate::cmd_necom::args::node_arg())
        .arg(crate::cmd_necom::args::name_list_arg())
        .arg(crate::cmd_necom::args::regex_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the distance command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

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
    let ids_name = super::common::match_names(&tree, args)?;
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
        }
    }

    let mut id_of = BTreeMap::new();
    let name_id_map = tree.get_name_id();
    let existing_names: std::collections::HashSet<&String> =
        name_id_map.keys().collect();
    for &id in &ids {
        let label = if let Some(name) =
            name_id_map
                .iter()
                .find_map(|(n, &i)| if i == id { Some(n) } else { None })
        {
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
