use super::Partition;
use crate::libs::phylo::tree::Tree;
use anyhow::Result;
use std::collections::HashMap;

/// Cut tree using Single Linkage (cut long branches).
pub fn cut_single_linkage(tree: &Tree, threshold: f64) -> Result<Partition> {
    let root = tree
        .get_root()
        .ok_or_else(|| anyhow::anyhow!("Tree has no root"))?;
    let mut part = Partition::new();
    let mut next_cluster_id = 0; // Starts from 0, incremented before use

    // Map NodeId -> ClusterId
    // This map is needed because when we assign a new cluster ID to a node,
    // we might not know if it's a leaf.
    // But Partition only stores Leaf assignments.

    // Stack: (node_id, cluster_id)
    // Root always starts a new cluster
    next_cluster_id += 1;
    let mut stack = vec![(root, next_cluster_id)];

    while let Some((u, cid)) = stack.pop() {
        let node = tree
            .get_node(u)
            .ok_or_else(|| anyhow::anyhow!("node {} not found", u))?;

        if node.children.is_empty() {
            part.assignment.insert(u, cid);
        } else {
            for &v in &node.children {
                let child_node = tree
                    .get_node(v)
                    .ok_or_else(|| anyhow::anyhow!("node {} not found", v))?;
                let len = child_node.finite_length();

                if len > threshold {
                    // Cut! v starts new cluster
                    next_cluster_id += 1;
                    stack.push((v, next_cluster_id));
                } else {
                    // v continues u's cluster
                    stack.push((v, cid));
                }
            }
        }
    }

    // Renumber clusters to be contiguous 1..K
    // Because "next_cluster_id" might have gaps if a cluster has no leaves?
    // Actually, with this logic, every created cluster ID is assigned to a node.
    // If that node is a leaf, it gets into partition.
    // If that node is internal but all its children are cut away, it has no leaves?
    // Yes, an internal node could be a cluster by itself but contain no leaves in partition map.
    // e.g. Root -> (len>T) Child. Root is cluster 1. Child is cluster 2.
    // Root has no leaves directly attached? If Root is internal node.
    // Then Cluster 1 is empty in terms of leaves.

    // So we need to normalize cluster IDs based on actual leaf assignments.
    // Sort old IDs before renumbering so the mapping is deterministic
    // (HashMap iteration order is random, which would otherwise make the
    // cluster_label assignment in `format_scan_rows` nondeterministic).
    let mut old_ids: Vec<usize> = part.assignment.values().copied().collect();
    old_ids.sort_unstable();
    old_ids.dedup();

    let old_to_new: HashMap<usize, usize> = old_ids
        .into_iter()
        .enumerate()
        .map(|(i, old_id)| (old_id, i + 1))
        .collect();

    for val in part.assignment.values_mut() {
        *val = old_to_new[val];
    }
    part.num_clusters = old_to_new.len();

    Ok(part)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::phylo::tree::Tree;
    use std::collections::HashMap;

    fn parse_tree(nwk: &str) -> Tree {
        Tree::from_newick(nwk).expect("valid newick")
    }

    fn cluster_names(part: &Partition, tree: &Tree) -> Vec<Vec<String>> {
        let mut groups: HashMap<usize, Vec<String>> = HashMap::new();
        for (&leaf_id, &cid) in &part.assignment {
            let name = tree
                .get_node(leaf_id)
                .and_then(|n| n.name.clone())
                .unwrap_or_else(|| format!("Node_{}", leaf_id));
            groups.entry(cid).or_default().push(name);
        }
        let mut clusters: Vec<Vec<String>> = groups.into_values().collect();
        for c in &mut clusters {
            c.sort();
        }
        clusters.sort();
        clusters
    }

    #[test]
    fn test_cut_single_linkage_all_singletons() {
        // Tree: ((A:1,B:1):1,C:1);
        // threshold < 1 cuts all edges.
        let tree = parse_tree("((A:1,B:1):1,C:1);");
        let part = cut_single_linkage(&tree, 0.5).unwrap();
        let mut clusters = cluster_names(&part, &tree);
        clusters.sort();
        assert_eq!(clusters, vec![vec!["A"], vec!["B"], vec!["C"]]);
    }

    #[test]
    fn test_cut_single_linkage_one_cluster() {
        // threshold >= 1 keeps all edges.
        let tree = parse_tree("((A:1,B:1):1,C:1);");
        let part = cut_single_linkage(&tree, 2.0).unwrap();
        let clusters = cluster_names(&part, &tree);
        assert_eq!(clusters, vec![vec!["A", "B", "C"]]);
    }

    #[test]
    fn test_cut_single_linkage_empty_tree() {
        let tree = Tree::new();
        assert!(cut_single_linkage(&tree, 1.0).is_err());
    }

    /// Regression test for nondeterministic cluster ID renumbering.
    ///
    /// `cut_single_linkage` renumbers cluster IDs based on HashMap iteration,
    /// which is random per-process. Sorting old IDs before renumbering (the
    /// fix) makes the output deterministic. This test runs the cut multiple
    /// times and asserts identical assignments every time.
    #[test]
    fn test_cut_single_linkage_renumbering_deterministic() {
        // Symmetric tree: all leaf edges have length 0.3, internal edges 0.1.
        // threshold=0.2 cuts all leaf edges -> 4 singleton clusters.
        let tree = parse_tree("((A:0.3,B:0.3):0.1,(C:0.3,D:0.3):0.1);");

        let baseline = cut_single_linkage(&tree, 0.2).unwrap();
        for run in 1..=20 {
            let part = cut_single_linkage(&tree, 0.2).unwrap();
            // Same leaf -> same cluster ID across runs.
            for (leaf, cid) in &baseline.assignment {
                assert_eq!(
                    part.assignment.get(leaf),
                    Some(cid),
                    "run {}: leaf {} reassigned from {} to {}",
                    run,
                    leaf,
                    cid,
                    part.assignment.get(leaf).unwrap_or(&0),
                );
            }
            assert_eq!(
                part.num_clusters, baseline.num_clusters,
                "run {}: num_clusters differs",
                run,
            );
        }
    }

    /// Regression test ensuring `cut_single_linkage` assigns contiguous
    /// cluster IDs 1..K with no gaps after renumbering.
    #[test]
    fn test_cut_single_linkage_contiguous_ids() {
        // Tree: ((A:0.3,B:0.3):0.1,(C:0.3,D:0.3):0.1);
        // threshold=0.2 -> 4 singletons, IDs must be 1, 2, 3, 4.
        let tree = parse_tree("((A:0.3,B:0.3):0.1,(C:0.3,D:0.3):0.1);");
        let part = cut_single_linkage(&tree, 0.2).unwrap();

        let mut ids: Vec<usize> = part.assignment.values().copied().collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids, vec![1, 2, 3, 4], "cluster IDs must be contiguous 1..4");
        assert_eq!(part.num_clusters, 4);
    }
}
