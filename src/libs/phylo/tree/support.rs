use super::Tree;
use crate::libs::phylo::node::NodeId;
use fixedbitset::FixedBitSet;
use std::collections::{BTreeMap, HashMap};

/// Build a map from leaf name to index (0..N-1).
///
/// Uses the first tree to establish the mapping. Duplicate leaf names are
/// rejected because they cannot be unambiguously matched across replicate trees.
pub fn build_leaf_map(tree: &Tree) -> anyhow::Result<BTreeMap<String, usize>> {
    let mut map = BTreeMap::new();
    let mut index = 0;

    let mut leaf_names = Vec::new();
    for node in &tree.nodes {
        if !node.deleted && node.is_leaf() {
            if let Some(name) = &node.name {
                leaf_names.push(name.clone());
            }
            // Unnamed leaves are skipped; they cannot be matched across replicate trees.
        }
    }

    leaf_names.sort();
    let len_before = leaf_names.len();
    leaf_names.dedup();
    let len_after = leaf_names.len();
    if len_after < len_before {
        anyhow::bail!(
            "{} duplicate leaf name(s) found; duplicate leaf names are not supported for support-value calculation",
            len_before - len_after
        );
    }

    for name in leaf_names {
        map.insert(name, index);
        index += 1;
    }

    Ok(map)
}

/// Compute bitsets for all nodes in the tree.
/// Returns a map NodeId -> FixedBitSet.
pub fn compute_all_bitsets(
    tree: &Tree,
    leaf_map: &BTreeMap<String, usize>,
) -> anyhow::Result<HashMap<NodeId, FixedBitSet>> {
    let num_leaves = leaf_map.len();
    let mut node_bitsets = HashMap::new();

    if let Some(root) = tree.get_root() {
        let traversal = tree.postorder(root);

        for id in traversal {
            let Some(node) = tree.get_node(id) else {
                continue;
            };
            let mut bitset = FixedBitSet::with_capacity(num_leaves);

            if node.is_leaf() {
                if let Some(name) = &node.name {
                    if let Some(&idx) = leaf_map.get(name) {
                        bitset.set(idx, true);
                    }
                }
            } else {
                for &child in &node.children {
                    if let Some(child_bs) = node_bitsets.get(&child) {
                        bitset.union_with(child_bs);
                    }
                }
            }
            node_bitsets.insert(id, bitset);
        }
    }

    Ok(node_bitsets)
}

/// Annotate internal nodes of `target` with support values from `counts`.
/// If `as_percent` is true, values are written as integer percentages of
/// `total_reps`, truncated toward zero.
///
/// By default the root node is skipped so that any existing root label is
/// preserved. Set `override_root` to `true` to also annotate the root.
pub fn annotate_support(
    target: &mut Tree,
    leaf_map: &BTreeMap<String, usize>,
    counts: &HashMap<FixedBitSet, usize>,
    total_reps: usize,
    as_percent: bool,
    override_root: bool,
) -> anyhow::Result<()> {
    let target_bitsets = compute_all_bitsets(target, leaf_map)?;
    for (id, bs) in target_bitsets {
        let node = target
            .get_node_mut(id)
            .ok_or_else(|| anyhow::anyhow!("invalid node id"))?;
        let is_root = node.parent.is_none();
        if !node.is_leaf() && (override_root || !is_root) {
            let count = counts.get(&bs).copied().unwrap_or(0);
            let label = if as_percent {
                match count
                    .checked_mul(100)
                    .and_then(|x| x.checked_div(total_reps))
                {
                    Some(v) => format!("{}", v),
                    None => "0".to_string(),
                }
            } else {
                format!("{}", count)
            };
            node.name = Some(label);
        }
    }
    Ok(())
}

/// Count clade frequencies from a list of replicate trees.
pub fn count_clades(
    trees: &[Tree],
    leaf_map: &BTreeMap<String, usize>,
) -> anyhow::Result<HashMap<FixedBitSet, usize>> {
    let mut counts = HashMap::new();

    for tree in trees {
        let bitsets = compute_all_bitsets(tree, leaf_map)?;

        for (id, bs) in bitsets {
            let Some(node) = tree.get_node(id) else {
                continue;
            };
            if !node.is_leaf() {
                *counts.entry(bs).or_insert(0) += 1;
            }
        }
    }

    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::phylo::tree::Tree;

    #[test]
    fn annotate_support_skips_root() {
        let target = Tree::from_newick("((A,B),(C,D));").unwrap();
        let replicate = Tree::from_newick("((A,B),(C,D));").unwrap();
        let leaf_map = build_leaf_map(&replicate).unwrap();
        let counts = count_clades(&[replicate], &leaf_map).unwrap();

        let mut annotated = target;
        annotate_support(&mut annotated, &leaf_map, &counts, 1, false, false).unwrap();

        let root = annotated.get_root().expect("root exists");
        assert!(
            annotated.get_node(root).unwrap().name.is_none(),
            "root node should not be annotated with a support value"
        );

        let internal_names: Vec<_> = annotated
            .nodes
            .iter()
            .filter(|n| !n.is_leaf() && n.parent.is_some())
            .filter_map(|n| n.name.clone())
            .collect();
        assert_eq!(internal_names, vec!["1", "1"]);
    }

    #[test]
    fn annotate_support_preserves_root_label() {
        let target = Tree::from_newick("((A,B),(C,D))Root;").unwrap();
        let replicate = Tree::from_newick("((A,B),(C,D));").unwrap();
        let leaf_map = build_leaf_map(&replicate).unwrap();
        let counts = count_clades(&[replicate], &leaf_map).unwrap();

        let mut annotated = target;
        annotate_support(&mut annotated, &leaf_map, &counts, 1, false, false).unwrap();

        let root = annotated.get_root().expect("root exists");
        assert_eq!(
            annotated.get_node(root).unwrap().name.as_deref(),
            Some("Root")
        );
    }

    #[test]
    fn annotate_support_percent_no_panic_on_zero_total() {
        // total_reps == 0 would divide by zero; the function should return 0
        // rather than panic.
        let target = Tree::from_newick("((A,B),(C,D));").unwrap();
        let replicate = Tree::from_newick("((A,B),(C,D));").unwrap();
        let leaf_map = build_leaf_map(&replicate).unwrap();
        let counts = count_clades(&[replicate], &leaf_map).unwrap();

        let mut annotated = target;
        annotate_support(&mut annotated, &leaf_map, &counts, 0, true, false).unwrap();

        let internal_names: Vec<_> = annotated
            .nodes
            .iter()
            .filter(|n| !n.is_leaf() && n.parent.is_some())
            .filter_map(|n| n.name.clone())
            .collect();
        assert_eq!(internal_names, vec!["0", "0"]);
    }

    #[test]
    fn annotate_support_override_root() {
        let target = Tree::from_newick("((A,B),(C,D))Root;").unwrap();
        let replicate = Tree::from_newick("((A,B),(C,D));").unwrap();
        let leaf_map = build_leaf_map(&replicate).unwrap();
        let counts = count_clades(&[replicate], &leaf_map).unwrap();

        let mut annotated = target;
        annotate_support(&mut annotated, &leaf_map, &counts, 1, false, true).unwrap();

        let root = annotated.get_root().expect("root exists");
        assert_eq!(
            annotated.get_node(root).unwrap().name.as_deref(),
            Some("1"),
            "root label should be overridden with support value"
        );
    }
}
