//! Shared helpers for `necom nwk` subcommands.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use anyhow::{anyhow, bail};
use clap::ArgMatches;
use necom::libs::phylo::node::{Node, NodeId};
use necom::libs::phylo::tree::io::escape_nhx_value;
use necom::libs::phylo::tree::stat::duplicate_names;
use necom::libs::phylo::tree::Tree;
use regex::RegexBuilder;
use std::fmt::Write as _;

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
pub(crate) fn format_label_columns(
    node: &Node,
    name: &str,
    columns: &[String],
) -> anyhow::Result<String> {
    let mut out = String::from(name);
    if columns.is_empty() {
        return Ok(out);
    }
    for column in columns {
        match column.as_str() {
            "dup" => {
                let _ = write!(out, "\t{}", name);
            }
            "taxid" => {
                let _ = write!(
                    out,
                    "\t{}",
                    node.get_property("T").map(|s| s.as_str()).unwrap_or("")
                );
            }
            "species" => {
                let _ = write!(
                    out,
                    "\t{}",
                    node.get_property("S").map(|s| s.as_str()).unwrap_or("")
                );
            }
            "full" => {
                let _ = write!(out, "\t");
                if let Some(p) = node.properties.as_ref().filter(|p| !p.is_empty()) {
                    let _ = write!(out, "[&&NHX");
                    for (k, v) in p {
                        if v.is_empty() {
                            let _ = write!(out, ":{}", k);
                        } else {
                            let _ = write!(out, ":{}={}", k, escape_nhx_value(v));
                        }
                    }
                    let _ = write!(out, "]");
                }
            }
            _ => bail!("unknown extra column: {}", column),
        }
    }
    Ok(out)
}

/// Return the value of a flag argument, defaulting to `false` when absent
/// or when the argument is not defined for the current command.
fn flag(args: &ArgMatches, id: &str) -> bool {
    args.try_contains_id(id).unwrap_or(false) && args.get_flag(id)
}

/// Warn when a duplicate name is being matched by name.
pub(crate) fn warn_duplicate_name(duplicates: &HashSet<String>, name: &str) {
    if duplicates.contains(name) {
        log::warn!(
            "duplicate node name '{}' matched multiple nodes; using one match",
            name
        );
    }
}

/// Collect IDs matched by `--node`, `--name-list`, and `--regex` using the
/// provided argument keys.
fn collect_name_selection(
    tree: &Tree,
    args: &ArgMatches,
    node_key: &str,
    name_list_key: &str,
    regex_key: &str,
) -> anyhow::Result<BTreeSet<NodeId>> {
    let id_of: BTreeMap<_, _> = tree.get_name_id();
    let duplicates = duplicate_names(tree);
    let mut ids = BTreeSet::new();

    if args.try_contains_id(node_key).unwrap_or(false) {
        if let Some(names) = args.get_many::<String>(node_key) {
            for name in names {
                if let Some(id) = id_of.get(name) {
                    warn_duplicate_name(&duplicates, name);
                    ids.insert(*id);
                } else {
                    log::warn!("node not found: {}", name);
                }
            }
        }
    }

    if args.try_contains_id(name_list_key).unwrap_or(false) {
        if let Some(file) = args.get_one::<String>(name_list_key) {
            for name in necom::libs::io::read_names::<Vec<String>>(file)?.iter() {
                if let Some(id) = id_of.get(name) {
                    warn_duplicate_name(&duplicates, name);
                    ids.insert(*id);
                } else {
                    log::warn!("name-list node not found: {}", name);
                }
            }
        }
    }

    if args.try_contains_id(regex_key).unwrap_or(false) {
        if let Some(regexes) = args.get_many::<String>(regex_key) {
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
    }

    Ok(ids)
}

/// Returns IDs of named nodes matching the name selection rules from CLI args.
///
/// When `default_to_all` is `true` and no name-based selector is provided,
/// all named nodes are returned. When `false`, an empty set is returned,
/// leaving the caller to decide what to do when nothing is selected.
pub(crate) fn match_names(
    tree: &Tree,
    args: &ArgMatches,
    default_to_all: bool,
) -> anyhow::Result<BTreeSet<NodeId>> {
    let mut ids = collect_name_selection(tree, args, "node", "name_list", "regex")?;

    let has_selector = args.try_contains_id("node").unwrap_or(false)
        || args.try_contains_id("name_list").unwrap_or(false)
        || args.try_contains_id("regex").unwrap_or(false);

    if !has_selector && default_to_all {
        ids = tree.get_name_id().values().cloned().collect();
    }

    // Include all descendants of internal nodes
    let is_descendants = flag(args, "descendants");

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
pub(crate) fn match_positions(
    tree: &Tree,
    args: &ArgMatches,
) -> anyhow::Result<BTreeSet<NodeId>> {
    let skip_internal = flag(args, "internal");
    let skip_leaf = flag(args, "leaf");

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
    let mut ids = collect_name_selection(tree, args, node_key, "name_list", "regex")?;

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

    #[test]
    fn format_label_columns_known_columns() {
        let mut node = Node::new(0);
        node.add_property("T", "9606");
        node.add_property("S", "Homo sapiens");
        let cols = vec![
            "dup".to_string(),
            "taxid".to_string(),
            "species".to_string(),
        ];
        assert_eq!(
            format_label_columns(&node, "Human", &cols).unwrap(),
            "Human\tHuman\t9606\tHomo sapiens"
        );
    }

    #[test]
    fn format_label_columns_unknown_column_errors() {
        let node = Node::new(0);
        let cols = vec!["unknown".to_string()];
        assert!(format_label_columns(&node, "A", &cols).is_err());
    }

    #[test]
    fn format_label_columns_full_escapes_nhx_values() {
        let mut node = Node::new(0);
        node.add_property("comment", "a]b\\c:d=e;f,g");
        let cols = vec!["full".to_string()];
        let out = format_label_columns(&node, "A", &cols).unwrap();
        assert_eq!(out, "A\t[&&NHX:comment=a\\]b\\\\c\\:d\\=e\\;f\\,g]");
    }

    #[test]
    fn format_label_columns_full_round_trips_through_parser() {
        let mut node = Node::new(0);
        node.add_property("comment", "a]b\\c:d=e;f,g");
        let cols = vec!["full".to_string()];
        let out = format_label_columns(&node, "A", &cols).unwrap();
        let newick = format!("{};", out.replace('\t', ""));
        let tree = Tree::from_newick(&newick).unwrap();
        let root = tree.get_root().unwrap();
        let parsed = tree.get_node(root).unwrap();
        assert_eq!(
            parsed.get_property("comment").map(|s| s.as_str()),
            Some("a]b\\c:d=e;f,g")
        );
    }
}
