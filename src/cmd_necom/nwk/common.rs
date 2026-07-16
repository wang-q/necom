//! Shared helpers for `necom nwk` subcommands.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use anyhow::anyhow;
use clap::ArgMatches;
use necom::libs::phylo::node::{Node, NodeId};
use necom::libs::phylo::tree::Tree;
use regex::RegexBuilder;

/// Parse a `--lca` argument value as two comma-separated names.
/// Returns `(&str, &str)` to avoid allocation; bails if the input does not
/// contain exactly one comma delimiting two non-empty names.
pub(crate) fn parse_lca_pair(lca: &str) -> anyhow::Result<(&str, &str)> {
    let (first, last) = lca.split_once(',').ok_or_else(|| {
        anyhow!(
            "--lca requires exactly two comma-separated names, got: {}",
            lca
        )
    })?;
    if first.is_empty() || last.is_empty() || last.contains(',') {
        anyhow::bail!(
            "--lca requires exactly two comma-separated names, got: {}",
            lca
        );
    }
    Ok((first, last))
}

/// Format a node's label with extra columns (`dup`, `taxid`, `species`, `full`).
pub(crate) fn format_label_columns(node: &Node, name: &str, columns: &[String]) -> String {
    let mut out = String::from(name);
    if columns.is_empty() {
        return out;
    }
    for column in columns {
        match column.as_str() {
            "dup" => out.push_str(&format!("\t{}", name)),
            "taxid" => out.push_str(&format!(
                "\t{}",
                node.get_property("T").map(|s| s.as_str()).unwrap_or("")
            )),
            "species" => out.push_str(&format!(
                "\t{}",
                node.get_property("S").map(|s| s.as_str()).unwrap_or("")
            )),
            "full" => {
                let comment = node
                    .properties
                    .as_ref()
                    .filter(|p| !p.is_empty())
                    .map(|p| {
                        let pairs: Vec<String> = p
                            .iter()
                            .map(|(k, v)| {
                                if v.is_empty() {
                                    format!(":{}", k)
                                } else {
                                    format!(":{}={}", k, v)
                                }
                            })
                            .collect();
                        format!("[&&NHX{}]", pairs.join(""))
                    })
                    .unwrap_or_default();
                out.push_str(&format!("\t{}", comment));
            }
            _ => {}
        }
    }
    out
}

/// Returns the set of node names that appear more than once in the tree.
fn duplicate_names(tree: &Tree) -> HashSet<String> {
    let Some(root) = tree.get_root() else {
        return HashSet::new();
    };

    let mut counts = HashMap::new();
    for id in tree.preorder(root) {
        if let Some(name) = tree.get_node(id).and_then(|n| n.name.as_deref()) {
            *counts.entry(name.to_string()).or_insert(0usize) += 1;
        }
    }

    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(name, _)| name)
        .collect()
}

/// Warn once for each duplicate name that is being matched by name.
fn warn_duplicate_name(duplicates: &HashSet<String>, name: &str) {
    if duplicates.contains(name) {
        log::warn!(
            "duplicate node name '{}' matched multiple nodes; using one match",
            name
        );
    }
}

/// Returns IDs of named nodes matching the name selection rules from CLI args.
pub(crate) fn match_names(tree: &Tree, args: &ArgMatches) -> anyhow::Result<BTreeSet<NodeId>> {
    // IDs with names
    let id_of: BTreeMap<_, _> = tree.get_name_id();
    let duplicates = duplicate_names(tree);

    // all matched IDs
    let mut ids = BTreeSet::new();

    // ids supplied by --node
    if args.try_contains_id("node").unwrap_or(false) {
        let names = args
            .get_many::<String>("node")
            .ok_or_else(|| anyhow!("missing --node values"))?;
        for name in names {
            if let Some(id) = id_of.get(name) {
                warn_duplicate_name(&duplicates, name);
                ids.insert(*id);
            } else {
                log::warn!("node not found: {}", name);
            }
        }
    }

    // ids supplied by --name-list
    if args.try_contains_id("name_list").unwrap_or(false) {
        let file = args
            .get_one::<String>("name_list")
            .ok_or_else(|| anyhow!("missing --name-list value"))?;
        for name in necom::libs::io::read_names::<Vec<String>>(file)?.iter() {
            if let Some(id) = id_of.get(name) {
                warn_duplicate_name(&duplicates, name);
                ids.insert(*id);
            } else {
                log::warn!("name-list node not found: {}", name);
            }
        }
    }

    // ids matched with --regex
    if args.try_contains_id("regex").unwrap_or(false) {
        let regexes = args
            .get_many::<String>("regex")
            .ok_or_else(|| anyhow!("missing --regex values"))?;
        for regex in regexes {
            let re = RegexBuilder::new(regex).case_insensitive(true).build()?;
            for (name, id) in id_of.iter() {
                if re.is_match(name) {
                    warn_duplicate_name(&duplicates, name);
                    ids.insert(*id);
                }
            }
        }
    }

    // Default is printing all named nodes
    let is_all = !(args.try_contains_id("node").unwrap_or(false)
        || args.try_contains_id("name_list").unwrap_or(false)
        || args.try_contains_id("regex").unwrap_or(false));

    if is_all {
        ids = id_of.values().cloned().collect();
    }

    // Include all descendants of internal nodes
    let is_descendants = if args.try_contains_id("descendants").is_ok() {
        args.get_flag("descendants")
    } else {
        false
    };

    if is_descendants {
        let internal_ids: Vec<NodeId> = ids
            .iter()
            .filter(|&&id| tree.get_node(id).map(|n| !n.is_leaf()).unwrap_or(false))
            .copied()
            .collect();
        for id in &internal_ids {
            for sid in tree.get_subtree(*id) {
                ids.insert(sid);
            }
        }
    }

    Ok(ids)
}

/// Returns IDs of nodes matching the position selection rules from CLI args.
pub(crate) fn match_positions(tree: &Tree, args: &ArgMatches) -> anyhow::Result<BTreeSet<NodeId>> {
    let skip_internal = if args.try_contains_id("internal").is_ok() {
        args.get_flag("internal")
    } else {
        false
    };
    let skip_leaf = if args.try_contains_id("leaf").is_ok() {
        args.get_flag("leaf")
    } else {
        false
    };

    // all matched IDs
    let mut ids = BTreeSet::new();

    let Some(root_id) = tree.get_root() else {
        anyhow::bail!("tree has no root; cannot select nodes");
    };
    let preorder_ids = tree.preorder(root_id);

    preorder_ids.iter().for_each(|id| {
        if let Some(node) = tree.get_node(*id) {
            if node.is_leaf() && !skip_leaf {
                ids.insert(*id);
            }
            if !node.is_leaf() && !skip_internal {
                ids.insert(*id);
            }
        }
    });

    Ok(ids)
}

/// Select nodes from `--node` / `--name-list` / `--regex` plus `--lca` pairs.
///
/// Unlike `match_names`, this never defaults to "all named nodes"; it only
/// returns nodes explicitly selected by the caller. This is important for
/// commands like `comment` and `rename`, where no selection means "do nothing".
pub(crate) fn match_nodes_and_lca(
    tree: &Tree,
    args: &ArgMatches,
    node_key: &str,
    lca_key: &str,
) -> anyhow::Result<BTreeSet<NodeId>> {
    let id_of: BTreeMap<_, _> = tree.get_name_id();
    let duplicates = duplicate_names(tree);
    let mut ids = BTreeSet::new();

    // Explicit --node names
    if args.try_contains_id(node_key).unwrap_or(false) {
        for name in args
            .get_many::<String>(node_key)
            .ok_or_else(|| anyhow::anyhow!("missing --{} values", node_key))?
        {
            if let Some(id) = id_of.get(name) {
                warn_duplicate_name(&duplicates, name);
                ids.insert(*id);
            } else {
                log::warn!("node not found: {}", name);
            }
        }
    }

    // --name-list file
    if args.try_contains_id("name_list").unwrap_or(false) {
        let file = args
            .get_one::<String>("name_list")
            .ok_or_else(|| anyhow::anyhow!("missing --name-list value"))?;
        for name in necom::libs::io::read_names::<Vec<String>>(file)?.iter() {
            if let Some(id) = id_of.get(name) {
                warn_duplicate_name(&duplicates, name);
                ids.insert(*id);
            } else {
                log::warn!("name-list node not found: {}", name);
            }
        }
    }

    // --regex patterns
    if args.try_contains_id("regex").unwrap_or(false) {
        for regex in args
            .get_many::<String>("regex")
            .ok_or_else(|| anyhow::anyhow!("missing --regex values"))?
        {
            let re = RegexBuilder::new(regex).case_insensitive(true).build()?;
            for (name, id) in id_of.iter() {
                if re.is_match(name) {
                    warn_duplicate_name(&duplicates, name);
                    ids.insert(*id);
                }
            }
        }
    }

    // --lca pairs
    if args.try_contains_id(lca_key).unwrap_or(false) {
        for lca in args
            .get_many::<String>(lca_key)
            .ok_or_else(|| anyhow::anyhow!("missing --{} values", lca_key))?
        {
            let (first, last) = parse_lca_pair(lca)?;
            warn_duplicate_name(&duplicates, first);
            warn_duplicate_name(&duplicates, last);
            match (id_of.get(first), id_of.get(last)) {
                (Some(id1), Some(id2)) => {
                    let ancestor = tree.get_common_ancestor(*id1, *id2)?;
                    ids.insert(ancestor);
                }
                _ => {
                    log::warn!("lca name not found in tree: {} / {}", first, last);
                }
            }
        }
    }

    Ok(ids)
}

/// Compute tree height for display, returning 0.0 when branch lengths are not used.
pub(crate) fn display_height(tree: &Tree, use_branch_length: bool) -> f64 {
    if use_branch_length {
        tree.get_root()
            .map(|r| tree.get_height(r, true))
            .unwrap_or(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lca_pair_valid() {
        assert_eq!(parse_lca_pair("a,b").unwrap(), ("a", "b"));
        assert_eq!(parse_lca_pair("foo,bar").unwrap(), ("foo", "bar"));
    }

    #[test]
    fn parse_lca_pair_invalid() {
        assert!(parse_lca_pair("a").is_err());
        assert!(parse_lca_pair("a,b,c").is_err());
        assert!(parse_lca_pair(",b").is_err());
        assert!(parse_lca_pair("a,").is_err());
        assert!(parse_lca_pair("").is_err());
    }

    #[test]
    fn duplicate_names_finds_duplicates() {
        let tree = Tree::from_newick("((A,A),(B,C));").unwrap();
        let dups = duplicate_names(&tree);
        assert!(dups.contains("A"));
        assert!(!dups.contains("B"));
        assert!(!dups.contains("C"));
    }

    #[test]
    fn duplicate_names_empty_for_unique() {
        let tree = Tree::from_newick("((A,B),(C,D));").unwrap();
        let dups = duplicate_names(&tree);
        assert!(dups.is_empty());
    }
}
