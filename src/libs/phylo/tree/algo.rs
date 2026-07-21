use super::Tree;
use crate::libs::pairmat::NamedMatrix;
use crate::libs::phylo::node::NodeId;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

/// Sort the children of each node by their name (label).
///
/// # Arguments
///
/// * `tree` - The tree to modify.
/// * `descending` - If true, sort in descending order (Z-A).
///
/// # Example
/// ```
/// use necom::libs::phylo::tree::Tree;
/// use necom::libs::phylo::tree::algo;
///
/// let mut tree = Tree::from_newick("(B,A);").unwrap();
/// algo::sort_by_name(&mut tree, false);
/// assert_eq!(tree.to_newick(), "(A,B);");
/// ```
pub fn sort_by_name(tree: &mut Tree, descending: bool) {
    if tree.is_empty() {
        return;
    }

    let Some(root) = tree.get_root() else {
        return;
    };
    let ids = tree.postorder(root);

    // Pre-collect names to avoid borrowing issues during sort
    let mut name_map: HashMap<NodeId, String> = HashMap::new();
    for &id in &ids {
        if let Some(node) = tree.get_node(id) {
            name_map.insert(id, node.name.clone().unwrap_or_default());
        }
    }

    for id in ids {
        let children = if let Some(node) = tree.get_node(id) {
            node.children.clone()
        } else {
            continue;
        };

        if children.is_empty() {
            continue;
        }

        let mut child_keys: HashMap<NodeId, String> = HashMap::new();
        for &child_id in &children {
            child_keys.insert(child_id, get_sort_key(&name_map, child_id));
        }

        if let Some(node) = tree.get_node_mut(id) {
            node.children.sort_by(|&a, &b| {
                let name_a = child_keys.get(&a).map(|s| s.as_str()).unwrap_or("");
                let name_b = child_keys.get(&b).map(|s| s.as_str()).unwrap_or("");

                if descending {
                    name_b.cmp(name_a)
                } else {
                    name_a.cmp(name_b)
                }
            });
        }

        if let Some(node) = tree.get_node(id) {
            if node.name.as_deref().unwrap_or("").is_empty() {
                if let Some(&first_child) = node.children.first() {
                    let child_key = get_sort_key(&name_map, first_child);
                    name_map.insert(id, child_key);
                }
            }
        }
    }
}

fn get_sort_key(name_map: &HashMap<NodeId, String>, id: NodeId) -> String {
    name_map.get(&id).cloned().unwrap_or_default()
}

/// Compute the number of descendants (including the node itself) for every node.
fn compute_subtree_sizes(tree: &Tree, root: NodeId) -> HashMap<NodeId, usize> {
    let mut size_map = HashMap::new();
    for &id in &tree.postorder(root) {
        let mut count = 0;
        if let Some(node) = tree.get_node(id) {
            if node.is_leaf() {
                count = 1;
            } else {
                count = 1;
                for child in &node.children {
                    count += size_map.get(child).unwrap_or(&0);
                }
            }
        }
        size_map.insert(id, count);
    }
    size_map
}

/// Sort the children of each node by the number of descendants (also known as ladderize).
///
/// # Arguments
///
/// * `tree` - The tree to modify.
/// * `descending` - If true, nodes with more descendants come first.
///
/// # Example
/// ```
/// use necom::libs::phylo::tree::Tree;
/// use necom::libs::phylo::tree::algo;
///
/// // ((A,B),C)
/// // (A,B) has 2 descendants (leaves), C has 1 descendant.
/// let mut tree = Tree::from_newick("((A,B),C);").unwrap();
///
/// // Ascending: C (1) < (A,B) (2)
/// algo::ladderize(&mut tree, false);
/// assert_eq!(tree.to_newick(), "(C,(A,B));");
/// ```
pub fn ladderize(tree: &mut Tree, descending: bool) {
    if tree.is_empty() {
        return;
    }

    let Some(root) = tree.get_root() else {
        return;
    };
    let ids = tree.levelorder(root);
    let size_map = compute_subtree_sizes(tree, root);

    for id in ids {
        if let Some(node) = tree.get_node_mut(id) {
            if node.children.is_empty() {
                continue;
            }

            // Stable sort so that an earlier alphanumeric order is preserved
            // within groups that have the same descendant count.
            if descending {
                node.children
                    .sort_by_key(|&c| Reverse(size_map.get(&c).copied().unwrap_or(0)));
            } else {
                node.children
                    .sort_by_key(|&c| size_map.get(&c).copied().unwrap_or(0));
            }
        }
    }
}

/// Sort the children of each node based on a list of names.
///
/// Nodes are ordered by the position of their descendants in the provided list.
/// If a node has multiple descendants in the list, the minimum position is used.
/// Nodes with no descendants in the list are placed at the end.
///
/// # Arguments
///
/// * `tree` - The tree to modify.
/// * `order_list` - A list of names defining the desired order.
///
/// # Example
/// ```
/// use necom::libs::phylo::tree::Tree;
/// use necom::libs::phylo::tree::algo;
///
/// let mut tree = Tree::from_newick("(A,B,C);").unwrap();
/// let order = vec!["C".to_string(), "B".to_string(), "A".to_string()];
/// algo::sort_by_list(&mut tree, &order);
/// assert_eq!(tree.to_newick(), "(C,B,A);");
/// ```
pub fn sort_by_list(tree: &mut Tree, order_list: &[String]) {
    if tree.is_empty() {
        return;
    }

    let Some(root) = tree.get_root() else {
        return;
    };

    // Map name -> position
    let mut pos_map: HashMap<String, usize> = HashMap::new();
    for (i, name) in order_list.iter().enumerate() {
        pos_map.insert(name.clone(), i);
    }

    let max_pos = order_list.len();
    let mut node_pos: HashMap<NodeId, usize> = HashMap::new();

    let ids = tree.postorder(root);
    for &id in &ids {
        let mut pos = max_pos;
        if let Some(node) = tree.get_node(id) {
            if let Some(name) = &node.name {
                if let Some(&p) = pos_map.get(name) {
                    pos = p;
                }
            }

            for &child in &node.children {
                if let Some(&child_p) = node_pos.get(&child) {
                    if child_p < pos {
                        pos = child_p;
                    }
                }
            }
        }
        node_pos.insert(id, pos);
    }

    let ids = tree.levelorder(root);
    for id in ids {
        if let Some(node) = tree.get_node_mut(id) {
            if node.children.is_empty() {
                continue;
            }
            node.children.sort_by(|a, b| {
                let pos_a = node_pos.get(a).unwrap_or(&max_pos);
                let pos_b = node_pos.get(b).unwrap_or(&max_pos);
                pos_a.cmp(pos_b)
            });
        }
    }
}

/// Sort the children of each node by the number of descendants, alternating direction at each level.
///
/// This produces a "balanced" look for the tree.
/// Level 0 (Root children): Ascending (Light -> Heavy)
/// Level 1: Descending (Heavy -> Light)
/// ...
pub fn deladderize(tree: &mut Tree) {
    if tree.is_empty() {
        return;
    }

    let Some(root) = tree.get_root() else {
        return;
    };

    // 1. Calculate descendant counts (shared helper used by ladderize).
    let size_map = compute_subtree_sizes(tree, root);

    // 2. Traversal with state
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((root, false)); // Start ascending

    while let Some((id, descending)) = queue.pop_front() {
        let children_ids = if let Some(node) = tree.get_node_mut(id) {
            if node.children.is_empty() {
                continue;
            }

            // Stable sort so that alternating levels preserve input order
            // within groups that have the same descendant count.
            if descending {
                node.children
                    .sort_by_key(|&c| Reverse(size_map.get(&c).copied().unwrap_or(0)));
            } else {
                node.children
                    .sort_by_key(|&c| size_map.get(&c).copied().unwrap_or(0));
            }
            node.children.clone()
        } else {
            continue;
        };

        for child in children_ids {
            queue.push_back((child, !descending));
        }
    }
}

/// Reorder leaves to minimize the sum of adjacent distances in the linear order.
///
/// Implements the optimal leaf ordering (OLO) algorithm of Bar-Joseph et al.
/// (2001). The tree topology is preserved; only the order of children at each
/// internal node is changed. Multifurcating trees are temporarily converted to
/// binary trees by left-associative pairing, the OLO is computed on that binary
/// tree, and the resulting leaf order is applied back to the original tree.
///
/// `dist` must be a symmetric distance matrix that covers every leaf in the
/// tree. Non-finite distances cause an error.
pub fn optimal_leaf_order(tree: &mut Tree, dist: &NamedMatrix) -> anyhow::Result<()> {
    if tree.is_empty() {
        return Ok(());
    }

    let Some(root) = tree.get_root() else {
        return Ok(());
    };

    let leaf_ids = current_leaf_order(tree, root);
    if leaf_ids.len() <= 1 {
        return Ok(());
    }

    let mut leaf_names: Vec<String> = Vec::with_capacity(leaf_ids.len());
    let mut seen_names: HashSet<String> = HashSet::new();
    for id in &leaf_ids {
        let name = tree
            .get_node(*id)
            .and_then(|n| n.name.clone())
            .ok_or_else(|| anyhow::anyhow!("leaf node {} has no name", id))?;
        if !seen_names.insert(name.clone()) {
            anyhow::bail!("duplicate leaf name: {}", name);
        }
        leaf_names.push(name);
    }

    for name in &leaf_names {
        if dist.get_index(name).is_none() {
            anyhow::bail!("distance matrix missing leaf: {}", name);
        }
    }

    let sorted_d = build_sorted_distance_matrix(&leaf_names, dist)?;
    let (sorted_z, cluster_ranges) = build_binary_linkage(tree, root, &leaf_ids)?;
    let must_swap = identify_swaps(&sorted_z, &sorted_d, &cluster_ranges);
    let new_order = apply_swaps(&sorted_z, &must_swap, leaf_ids.len());

    let ordered_names: Vec<String> =
        new_order.iter().map(|&i| leaf_names[i].clone()).collect();
    sort_by_list(tree, &ordered_names);

    Ok(())
}

/// Return leaf IDs in their current left-to-right order.
fn current_leaf_order(tree: &Tree, root: NodeId) -> Vec<NodeId> {
    let mut order = Vec::new();
    fn collect(tree: &Tree, id: NodeId, order: &mut Vec<NodeId>) {
        let Some(node) = tree.get_node(id) else {
            return;
        };
        if node.is_leaf() {
            order.push(id);
            return;
        }
        for &child in &node.children {
            collect(tree, child, order);
        }
    }
    collect(tree, root, &mut order);
    order
}

/// Extract the distance matrix reordered to match the current leaf order.
fn build_sorted_distance_matrix(
    leaf_names: &[String],
    dist: &NamedMatrix,
) -> anyhow::Result<Vec<Vec<f32>>> {
    let n = leaf_names.len();
    let mut indices = Vec::with_capacity(n);
    for name in leaf_names {
        let idx = dist
            .get_index(name)
            .ok_or_else(|| anyhow::anyhow!("distance matrix missing leaf: {}", name))?;
        indices.push(idx);
    }

    let mut sorted_d = vec![vec![0.0f32; n]; n];
    for i in 0..n {
        for j in 0..n {
            let value = dist.get(indices[i], indices[j]);
            if !value.is_finite() {
                anyhow::bail!(
                    "non-finite distance between '{}' and '{}'",
                    leaf_names[i],
                    leaf_names[j]
                );
            }
            sorted_d[i][j] = value;
        }
    }
    Ok(sorted_d)
}

/// A binary internal node used during OLO.
#[derive(Debug)]
struct BinaryNode {
    left: usize,
    right: usize,
    size: usize,
}

/// Build a binary linkage representation of the tree.
///
/// Returns `sorted_z` (rows are `[left, right, size]`) and `cluster_ranges`
/// (each row is `[start, end)` over the leaf indices). Internal node `i` in
/// `sorted_z` has global index `n_leaves + i`.
#[allow(clippy::type_complexity)]
fn build_binary_linkage(
    tree: &Tree,
    root: NodeId,
    leaf_ids: &[NodeId],
) -> anyhow::Result<(Vec<[usize; 3]>, Vec<[usize; 2]>)> {
    let n_leaves = leaf_ids.len();
    let mut leaf_index: HashMap<NodeId, usize> = HashMap::with_capacity(n_leaves);
    for (i, &id) in leaf_ids.iter().enumerate() {
        leaf_index.insert(id, i);
    }

    let mut internals: Vec<BinaryNode> = Vec::with_capacity(n_leaves.saturating_sub(1));

    fn build(
        tree: &Tree,
        id: NodeId,
        leaf_index: &HashMap<NodeId, usize>,
        internals: &mut Vec<BinaryNode>,
    ) -> anyhow::Result<usize> {
        let Some(node) = tree.get_node(id) else {
            anyhow::bail!("node {} not found", id);
        };

        if node.is_leaf() {
            return Ok(leaf_index[&id]);
        }

        if node.children.is_empty() {
            anyhow::bail!("internal node {} has no children", id);
        }

        let mut child_roots: Vec<usize> = Vec::with_capacity(node.children.len());
        for &child in &node.children {
            child_roots.push(build(tree, child, leaf_index, internals)?);
        }

        // Combine children left-associatively, preserving their current order.
        let mut current = child_roots[0];
        let n_leaves = leaf_index.len();
        for &next in &child_roots[1..] {
            let current_size = if current < n_leaves {
                1
            } else {
                internals[current - n_leaves].size
            };
            let next_size = if next < n_leaves {
                1
            } else {
                internals[next - n_leaves].size
            };
            internals.push(BinaryNode {
                left: current,
                right: next,
                size: current_size + next_size,
            });
            current = n_leaves + internals.len() - 1;
        }
        Ok(current)
    }

    build(tree, root, &leaf_index, &mut internals)?;

    let n_clusters = n_leaves + internals.len();
    let mut cluster_ranges: Vec<[usize; 2]> = vec![[0, 0]; n_clusters];
    for (i, range) in cluster_ranges.iter_mut().enumerate().take(n_leaves) {
        *range = [i, i + 1];
    }

    let mut sorted_z: Vec<[usize; 3]> = Vec::with_capacity(internals.len());
    for (i, node) in internals.iter().enumerate() {
        let global = n_leaves + i;
        cluster_ranges[global] =
            [cluster_ranges[node.left][0], cluster_ranges[node.right][1]];
        sorted_z.push([node.left, node.right, node.size]);
    }

    Ok((sorted_z, cluster_ranges))
}

/// OLO dynamic programming: return whether each binary internal node must swap
/// its children.
fn identify_swaps(
    sorted_z: &[[usize; 3]],
    sorted_d: &[Vec<f32>],
    cluster_ranges: &[[usize; 2]],
) -> Vec<bool> {
    let n_points = sorted_d.len();
    if sorted_z.is_empty() {
        return Vec::new();
    }

    let mut m = vec![vec![0.0f32; n_points]; n_points];
    let mut swap_status = vec![vec![[0u8, 0u8]; n_points]; n_points];
    let mut must_swap = vec![false; sorted_z.len()];

    // SciPy uses 2^30 as a sufficiently large finite value.
    const INF: f32 = 1_073_741_824.0;

    for z in sorted_z.iter() {
        let v_l = z[0];
        let v_r = z[1];

        let [v_l_min, v_l_max] = cluster_ranges[v_l];
        let [v_r_min, v_r_max] = cluster_ranges[v_r];

        let (u_clusters, m_clusters) = if v_l < n_points {
            (vec![v_l], vec![v_l])
        } else {
            let left_z = &sorted_z[v_l - n_points];
            (vec![left_z[0], left_z[1]], vec![left_z[1], left_z[0]])
        };

        let (w_clusters, k_clusters) = if v_r < n_points {
            (vec![v_r], vec![v_r])
        } else {
            let right_z = &sorted_z[v_r - n_points];
            (vec![right_z[1], right_z[0]], vec![right_z[0], right_z[1]])
        };

        for swap_l in 0..u_clusters.len() {
            let u_min = cluster_ranges[u_clusters[swap_l]][0];
            let u_max = cluster_ranges[u_clusters[swap_l]][1];
            let m_min = cluster_ranges[m_clusters[swap_l]][0];
            let m_max = cluster_ranges[m_clusters[swap_l]][1];

            for swap_r in 0..w_clusters.len() {
                let w_min = cluster_ranges[w_clusters[swap_r]][0];
                let w_max = cluster_ranges[w_clusters[swap_r]][1];
                let k_min = cluster_ranges[k_clusters[swap_r]][0];
                let k_max = cluster_ranges[k_clusters[swap_r]][1];

                let mut min_km_dist = INF;
                for row in sorted_d.iter().take(m_max).skip(m_min) {
                    for &val in row.iter().take(k_max).skip(k_min) {
                        if val < min_km_dist {
                            min_km_dist = val;
                        }
                    }
                }

                for u in u_min..u_max {
                    let mut m_sorted: Vec<(usize, f32)> =
                        (m_min..m_max).map(|mi| (mi, m[mi][u])).collect();
                    m_sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

                    for w in w_min..w_max {
                        let mut k_sorted: Vec<(usize, f32)> =
                            (k_min..k_max).map(|ki| (ki, m[ki][w])).collect();
                        k_sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

                        let mut cur_min_m = INF;

                        for &(m_idx, m_u_m) in &m_sorted {
                            let k0_val =
                                k_sorted.first().map(|(_, v)| *v).unwrap_or(INF);
                            if m_u_m + k0_val + min_km_dist >= cur_min_m {
                                break;
                            }
                            for &(k_idx, m_w_k) in &k_sorted {
                                if m_u_m + m_w_k + min_km_dist >= cur_min_m {
                                    break;
                                }
                                let current_m = m_u_m + m_w_k + sorted_d[m_idx][k_idx];
                                if current_m < cur_min_m {
                                    cur_min_m = current_m;
                                }
                            }
                        }

                        m[u][w] = cur_min_m;
                        m[w][u] = cur_min_m;
                        swap_status[u][w] = [swap_l as u8, swap_r as u8];
                        swap_status[w][u] = [swap_l as u8, swap_r as u8];
                    }
                }
            }
        }

        let mut cur_min_m = INF;
        let mut best_u = v_l_min;
        let mut best_w = v_r_min;
        for (u, row) in m.iter().enumerate().take(v_l_max).skip(v_l_min) {
            for (w, &val) in row.iter().enumerate().take(v_r_max).skip(v_r_min) {
                if val < cur_min_m {
                    cur_min_m = val;
                    best_u = u;
                    best_w = w;
                }
            }
        }

        if v_l >= n_points {
            must_swap[v_l - n_points] = swap_status[best_u][best_w][0] == 1;
        }
        if v_r >= n_points {
            must_swap[v_r - n_points] = swap_status[best_u][best_w][1] == 1;
        }
    }

    must_swap
}

/// Derive the new leaf order from the binary linkage and swap decisions.
fn apply_swaps(
    sorted_z: &[[usize; 3]],
    must_swap: &[bool],
    n_leaves: usize,
) -> Vec<usize> {
    if sorted_z.is_empty() {
        return (0..n_leaves).collect();
    }

    let mut order = Vec::with_capacity(n_leaves);
    collect_leaf_order(
        sorted_z,
        must_swap,
        n_leaves,
        n_leaves + sorted_z.len() - 1,
        false,
        &mut order,
    );
    order
}

fn collect_leaf_order(
    sorted_z: &[[usize; 3]],
    must_swap: &[bool],
    n_leaves: usize,
    node: usize,
    parent_parity: bool,
    order: &mut Vec<usize>,
) {
    if node < n_leaves {
        order.push(node);
        return;
    }
    let i = node - n_leaves;
    let left = sorted_z[i][0];
    let right = sorted_z[i][1];
    let effective_swap = must_swap[i] ^ parent_parity;

    let (first, second) = if effective_swap {
        (right, left)
    } else {
        (left, right)
    };

    collect_leaf_order(sorted_z, must_swap, n_leaves, first, effective_swap, order);
    collect_leaf_order(sorted_z, must_swap, n_leaves, second, effective_swap, order);
}

/// Compute the set of node IDs to keep when inverting a prune around `targets`.
///
/// Returns the union of `targets`, all descendants of any target, and all
/// ancestors of any target. Nodes not in the returned set should be removed.
pub fn compute_keep_set<I>(tree: &Tree, targets: I) -> HashSet<NodeId>
where
    I: IntoIterator<Item = NodeId>,
{
    let mut keep = HashSet::new();
    let Some(root) = tree.get_root() else {
        return keep;
    };

    let target_set: HashSet<NodeId> = targets.into_iter().collect();
    let mut is_in_clade = HashSet::new();

    // Pass 1: Downward propagation (descendants of targets).
    // levelorder visits parents before children, so `is_in_clade` propagates.
    let all_nodes = tree.levelorder(root);
    for &id in &all_nodes {
        let mut kept = target_set.contains(&id);
        if !kept {
            if let Some(node) = tree.get_node(id) {
                if let Some(parent) = node.parent {
                    if is_in_clade.contains(&parent) {
                        kept = true;
                    }
                }
            }
        }
        if kept {
            is_in_clade.insert(id);
            keep.insert(id);
        }
    }

    // Pass 2: Upward propagation (ancestors of kept nodes).
    // Iterate in reverse so children are visited before their parents.
    for &id in all_nodes.iter().rev() {
        if keep.contains(&id) {
            if let Some(node) = tree.get_node(id) {
                if let Some(parent) = node.parent {
                    keep.insert(parent);
                }
            }
        }
    }

    keep
}

/// Remove `to_remove` nodes from `tree`, then clean up internal nodes that
/// became leaves and collapse degree-2 nodes (single-child internals).
///
/// `to_remove` may include both leaves and internal nodes; removal is recursive
/// (subtrees are detached with their parent).
///
/// # Note
///
/// This function calls `tree.compact()` before returning, so all previously
/// held `NodeId`s are invalidated. Callers that need to keep using node IDs
/// must do so before calling this function.
pub fn prune_nodes(tree: &mut Tree, to_remove: Vec<NodeId>) -> anyhow::Result<()> {
    // 1. Snapshot internal nodes before pruning, so we can detect those that
    //    become leaves after removal.
    let mut old_internals = Vec::new();
    if let Some(root) = tree.get_root() {
        let all_nodes = tree.levelorder(root);
        for id in all_nodes {
            if let Some(node) = tree.get_node(id) {
                if !node.children.is_empty() {
                    old_internals.push(id);
                }
            }
        }
    }

    // 2. Remove the requested nodes.
    for id in to_remove {
        tree.remove_node(id, true);
    }

    // 3. Clean up internals that became leaves (reverse so deeper nodes first).
    for id in old_internals.into_iter().rev() {
        if let Some(node) = tree.get_node(id) {
            if node.children.is_empty() {
                tree.remove_node(id, true);
            }
        }
    }

    // 4. Collapse degree-2 nodes (post-order so children are visited first).
    if let Some(root) = tree.get_root() {
        let nodes = tree.postorder(root);
        for id in nodes {
            if let Some(node) = tree.get_node(id) {
                if node.children.len() == 1 {
                    if tree.get_root() == Some(id) {
                        // Root with a single child: promote the child to root.
                        let child_id = node.children[0];
                        tree.set_root(child_id)?;
                        tree.remove_node(id, false);
                    } else {
                        tree.collapse_node(id)?;
                    }
                }
            }
        }
    }

    // 5. Remove soft-deleted nodes and reclaim arena slots. This invalidates
    //    any NodeIds held outside this function.
    tree.compact();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::phylo::tree::Tree;

    #[test]
    fn test_sort_by_name() {
        let mut tree = Tree::new();
        let root = tree.add_node();
        let _ = tree.set_root(root);

        let c1 = tree.add_node();
        tree.get_node_mut(c1).unwrap().name = Some("C".to_string());
        let c2 = tree.add_node();
        tree.get_node_mut(c2).unwrap().name = Some("A".to_string());
        let c3 = tree.add_node();
        tree.get_node_mut(c3).unwrap().name = Some("B".to_string());

        tree.add_child(root, c1).unwrap();
        tree.add_child(root, c2).unwrap();
        tree.add_child(root, c3).unwrap();

        // Before sort: C, A, B
        sort_by_name(&mut tree, false);

        let children = &tree.get_node(root).unwrap().children;
        assert_eq!(children.len(), 3);
        assert_eq!(
            tree.get_node(children[0]).unwrap().name.as_deref(),
            Some("A")
        );
        assert_eq!(
            tree.get_node(children[1]).unwrap().name.as_deref(),
            Some("B")
        );
        assert_eq!(
            tree.get_node(children[2]).unwrap().name.as_deref(),
            Some("C")
        );
    }

    #[test]
    fn test_ladderize() {
        let mut tree = Tree::new();
        let root = tree.add_node();
        let _ = tree.set_root(root);

        // Child 1: Leaf (Size 1)
        let c1 = tree.add_node();

        // Child 2: Has 2 children (Size 3)
        let c2 = tree.add_node();
        let c2_1 = tree.add_node();
        let c2_2 = tree.add_node();
        tree.add_child(c2, c2_1).unwrap();
        tree.add_child(c2, c2_2).unwrap();

        tree.add_child(root, c1).unwrap();
        tree.add_child(root, c2).unwrap();

        // Sort ascending (smallest first) -> c1, c2
        ladderize(&mut tree, false);
        let children = &tree.get_node(root).unwrap().children;
        assert_eq!(children[0], c1);
        assert_eq!(children[1], c2);

        // Sort descending (largest first) -> c2, c1
        ladderize(&mut tree, true);
        let children = &tree.get_node(root).unwrap().children;
        assert_eq!(children[0], c2);
        assert_eq!(children[1], c1);
    }

    #[test]
    fn test_sort_by_list() {
        let mut tree = Tree::new();
        let root = tree.add_node();
        let _ = tree.set_root(root);

        // ( (A, B), C )
        // Let's create a structure:
        // root -> n1 (children A, B)
        // root -> C
        let n1 = tree.add_node();
        let c = tree.add_node();
        tree.get_node_mut(c).unwrap().name = Some("C".to_string());

        tree.add_child(root, n1).unwrap();
        tree.add_child(root, c).unwrap();

        let a = tree.add_node();
        tree.get_node_mut(a).unwrap().name = Some("A".to_string());
        let b = tree.add_node();
        tree.get_node_mut(b).unwrap().name = Some("B".to_string());

        tree.add_child(n1, a).unwrap();
        tree.add_child(n1, b).unwrap();

        // Current order of root children: n1, c
        // Current order of n1 children: a, b

        // Target list: ["C", "B", "A"]
        // Expected:
        // root children: C (pos 0), n1 (pos min(pos(B)=1, pos(A)=2) = 1) -> (C, n1)
        // n1 children: B (pos 1), A (pos 2) -> (B, A)

        let order = vec!["C".to_string(), "B".to_string(), "A".to_string()];
        sort_by_list(&mut tree, &order);

        let root_children = &tree.get_node(root).unwrap().children;
        assert_eq!(root_children[0], c);
        assert_eq!(root_children[1], n1);

        let n1_children = &tree.get_node(n1).unwrap().children;
        assert_eq!(n1_children[0], b);
        assert_eq!(n1_children[1], a);
    }

    #[test]
    fn test_sort_by_list_comprehensive() {
        // Case 1: Simple case with only leaf nodes
        let newick = "(A,B,C);";
        let mut tree = Tree::from_newick(newick).unwrap();
        sort_by_list(
            &mut tree,
            &["C".to_string(), "B".to_string(), "A".to_string()],
        );
        assert_eq!(tree.to_newick(), "(C,B,A);");

        // Case 2: Case with internal nodes
        let newick = "((A,B),(C,D));";
        let mut tree = Tree::from_newick(newick).unwrap();
        sort_by_list(
            &mut tree,
            &["C".to_string(), "B".to_string(), "A".to_string()],
        );
        assert_eq!(tree.to_newick(), "((C,D),(B,A));");

        // Case 3: Case with internal nodes and names
        let newick = "((A,B)X,(C,D)Y);";
        let mut tree = Tree::from_newick(newick).unwrap();
        sort_by_list(
            &mut tree,
            &["C".to_string(), "B".to_string(), "A".to_string()],
        );
        assert_eq!(tree.to_newick(), "((C,D)Y,(B,A)X);");

        // Case 4: Case with unlisted nodes
        let newick = "((A,B),(C,E));";
        let mut tree = Tree::from_newick(newick).unwrap();
        sort_by_list(&mut tree, &["C".to_string(), "B".to_string()]);
        assert_eq!(tree.to_newick(), "((C,E),(B,A));");
    }

    #[test]
    fn test_sort_by_list_preserves_order_for_ties() {
        // When two children have the same position in the list (or are both
        // unlisted), stable sorting should preserve their original order.
        let newick = "((Z,Y),(X,W));";
        let mut tree = Tree::from_newick(newick).unwrap();
        sort_by_list(&mut tree, &["A".to_string(), "B".to_string()]);
        // All leaves are unlisted, so their relative order within each clade
        // should be unchanged.
        assert_eq!(tree.to_newick(), "((Z,Y),(X,W));");
    }

    #[test]
    fn test_deladderize() {
        let mut tree = Tree::new();
        let root = tree.add_node();
        let _ = tree.set_root(root);

        // Structure: ((A,B),(C,(D,E)),F)
        // Sizes:
        // A,B,C,D,E,F (leaves) = 1
        // (A,B) = 1 + 1 + 1 = 3
        // (D,E) = 1 + 1 + 1 = 3
        // (C,(D,E)) = 1 + 1 + 3 = 5
        // Root children:
        // 1. (A,B) - size 3
        // 2. (C,(D,E)) - size 5
        // 3. F - size 1

        let f = tree.add_node();
        tree.get_node_mut(f).unwrap().name = Some("F".to_string());

        let ab = tree.add_node();
        let a = tree.add_node();
        tree.get_node_mut(a).unwrap().name = Some("A".to_string());
        let b = tree.add_node();
        tree.get_node_mut(b).unwrap().name = Some("B".to_string());
        tree.add_child(ab, a).unwrap();
        tree.add_child(ab, b).unwrap();

        let cde = tree.add_node();
        let c = tree.add_node();
        tree.get_node_mut(c).unwrap().name = Some("C".to_string());
        let de = tree.add_node();
        let d = tree.add_node();
        tree.get_node_mut(d).unwrap().name = Some("D".to_string());
        let e = tree.add_node();
        tree.get_node_mut(e).unwrap().name = Some("E".to_string());
        tree.add_child(de, d).unwrap();
        tree.add_child(de, e).unwrap();

        tree.add_child(cde, c).unwrap(); // Add C first
        tree.add_child(cde, de).unwrap(); // Add (D,E) second

        tree.add_child(root, ab).unwrap();
        tree.add_child(root, cde).unwrap();
        tree.add_child(root, f).unwrap();

        // Run deladderize
        // Level 0 (Root children): Ascending -> F (1), (A,B) (3), (C,(D,E)) (5)
        // Level 1 (Children of Level 0 nodes): Descending
        // - F: no children
        // - (A,B): A(1), B(1) -> Equal, keep order (A,B)
        // - (C,(D,E)): C(1), (D,E)(3) -> Descending -> (D,E), C
        // Level 2 (Children of Level 1 nodes): Ascending
        // - (D,E): D(1), E(1) -> Equal, keep order (D,E)

        deladderize(&mut tree);

        let root_children = &tree.get_node(root).unwrap().children;
        assert_eq!(root_children.len(), 3);
        assert_eq!(root_children[0], f);
        assert_eq!(root_children[1], ab);
        assert_eq!(root_children[2], cde);

        let cde_children = &tree.get_node(cde).unwrap().children;
        assert_eq!(cde_children[0], de); // Larger one first (descending)
        assert_eq!(cde_children[1], c);
    }

    #[test]
    fn test_ladderize_is_stable() {
        // All children have the same size, so stable sort must preserve input order.
        let mut tree = Tree::from_newick("(Z,Y,X);").unwrap();
        ladderize(&mut tree, false);
        assert_eq!(tree.to_newick(), "(Z,Y,X);");
    }

    #[test]
    fn test_sort_by_name_then_ladderize() {
        // Leaves in reverse alphabetical order. sort_by_name puts them in
        // alphabetical order; ladderize is stable and preserves that order
        // because all leaves have the same descendant count.
        let mut tree = Tree::from_newick("(D,C,B,A);").unwrap();
        sort_by_name(&mut tree, false);
        ladderize(&mut tree, false);
        assert_eq!(tree.to_newick(), "(A,B,C,D);");
    }

    #[test]
    fn prune_nodes_empty_does_nothing() {
        let mut tree = Tree::from_newick("(A,B,(C,D));").unwrap();
        prune_nodes(&mut tree, Vec::new()).unwrap();
        assert_eq!(tree.to_newick(), "(A,B,(C,D));");
    }

    #[test]
    fn prune_nodes_remove_root_yields_empty_tree() {
        let mut tree = Tree::from_newick("(A,B,(C,D));").unwrap();
        let root = tree.get_root().unwrap();
        prune_nodes(&mut tree, vec![root]).unwrap();
        assert!(tree.is_empty());
        assert!(tree.to_newick().is_empty());
    }

    #[test]
    fn prune_nodes_remove_all_leaves_yields_empty_tree() {
        let mut tree = Tree::from_newick("(A,B,(C,D));").unwrap();
        let leaves: Vec<_> = tree
            .nodes
            .iter()
            .filter(|n| !n.deleted && n.is_leaf())
            .map(|n| n.id)
            .collect();
        prune_nodes(&mut tree, leaves).unwrap();
        assert!(tree.is_empty());
    }

    #[test]
    fn prune_nodes_keep_single_leaf() {
        // Invert-style pruning that leaves a single leaf should produce a
        // one-node tree without panic.
        let mut tree = Tree::from_newick("(A,B,(C,D));").unwrap();
        let target = tree.get_node_by_name("A").unwrap();
        let keep = compute_keep_set(&tree, [target]);
        let root = tree.get_root().unwrap();
        let to_remove: Vec<_> = tree
            .levelorder(root)
            .into_iter()
            .filter(|id| !keep.contains(id))
            .collect();
        prune_nodes(&mut tree, to_remove).unwrap();
        assert_eq!(tree.to_newick(), "A;");
    }

    #[test]
    fn test_optimal_leaf_order_simple() {
        // Tree: ((A,(B,C)),D). Distances favour keeping B-C and A-D adjacent.
        let mut tree = Tree::from_newick("((A,(B,C)),D);").unwrap();
        let names = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ];
        // Condensed upper-triangle order: AB, AC, AD, BC, BD, CD.
        let values = vec![10.0, 10.0, 1.0, 1.0, 10.0, 10.0];
        let dist = NamedMatrix::new_from_values(names, values).unwrap();

        optimal_leaf_order(&mut tree, &dist).unwrap();
        assert_eq!(tree.to_newick(), "(((B,C),A),D);");
    }

    #[test]
    fn test_optimal_leaf_order_multifurcating() {
        // Tree: (A,B,C,D). Distances favour A-D and B-C adjacency.
        let mut tree = Tree::from_newick("(A,B,C,D);").unwrap();
        let names = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ];
        // AB=10, AC=10, AD=1, BC=1, BD=10, CD=10.
        let values = vec![10.0, 10.0, 1.0, 1.0, 10.0, 10.0];
        let dist = NamedMatrix::new_from_values(names, values).unwrap();

        optimal_leaf_order(&mut tree, &dist).unwrap();
        // The left-associative binary conversion keeps B and C adjacent and
        // places A next to D.
        assert_eq!(tree.to_newick(), "(C,B,A,D);");
    }

    #[test]
    fn test_optimal_leaf_order_missing_leaf_errors() {
        let mut tree = Tree::from_newick("(A,B,C);").unwrap();
        let names = vec!["A".to_string(), "B".to_string(), "D".to_string()];
        let values = vec![1.0, 1.0, 1.0];
        let dist = NamedMatrix::new_from_values(names, values).unwrap();

        let err = optimal_leaf_order(&mut tree, &dist).unwrap_err();
        assert!(err.to_string().contains("distance matrix missing leaf"));
        assert!(err.to_string().contains("C"));
    }

    #[test]
    fn test_optimal_leaf_order_non_finite_errors() {
        let mut tree = Tree::from_newick("(A,B);").unwrap();
        let names = vec!["A".to_string(), "B".to_string()];
        let values = vec![f32::NAN];
        let dist = NamedMatrix::new_from_values(names, values).unwrap();

        let err = optimal_leaf_order(&mut tree, &dist).unwrap_err();
        assert!(err.to_string().contains("non-finite distance"));
    }

    // ========================================================================
    // SciPy parity tests for `optimal_leaf_order`.
    //
    // Mirrors `scipy.cluster.hierarchy.tests.test_hierarchy::
    // test_optimal_leaf_ordering` (line 1214-1219). We rebuild the input tree
    // via our `linkage` + `to_tree` (which already match SciPy's linkage
    // output), apply `optimal_leaf_order`, and compare the resulting leaf
    // order against SciPy's `linkage_ytdist_single_olo` /
    // `linkage_X_ward_olo` decoded leaf sequences. OLO does not distinguish
    // left-right mirroring, so both directions are accepted.
    // ========================================================================

    /// Build a tree from a condensed distance vector using the given linkage
    /// method, then return `(tree, dist_matrix)` for OLO.
    fn build_tree_for_olo(
        names: Vec<String>,
        condensed: Vec<f32>,
        method: crate::libs::clust::hier::Method,
    ) -> (Tree, NamedMatrix) {
        let dist = NamedMatrix::new_from_values(names.clone(), condensed).unwrap();
        let steps = crate::libs::clust::hier::linkage(&dist, method).unwrap();
        let tree = crate::libs::clust::hier::to_tree(&steps, &names).unwrap();
        (tree, dist)
    }

    /// Extract the leaf names in left-to-right order from a tree.
    fn leaf_name_order(tree: &Tree) -> Vec<String> {
        tree.get_leaf_names()
            .into_iter()
            .map(|opt| opt.expect("leaf should have a name"))
            .collect()
    }

    #[test]
    fn test_scipy_olo_ytdist() {
        // SciPy `linkage_ytdist_single_olo = [[5,2,138,2],[4,3,219,2],
        // [7,0,255,3],[1,8,268,4],[6,9,295,6]]` decodes to the tree
        // ((5,2),(1,((4,3),0))) with left-to-right leaves [5,2,1,4,3,0].
        let names: Vec<String> = (0..6).map(|i| i.to_string()).collect();
        let condensed = vec![
            662.0, 877.0, 255.0, 412.0, 996.0, // (0,*) pairs
            295.0, 468.0, 268.0, 400.0, // (1,*) pairs
            754.0, 564.0, 138.0, // (2,*) pairs
            219.0, 869.0, // (3,*) pairs
            669.0, // (4,5)
        ];
        let (mut tree, dist) = build_tree_for_olo(
            names,
            condensed,
            crate::libs::clust::hier::Method::Single,
        );

        optimal_leaf_order(&mut tree, &dist).unwrap();

        let order = leaf_name_order(&tree);
        let expected = vec!["5", "2", "1", "4", "3", "0"];
        let expected_rev: Vec<&str> = expected.iter().rev().copied().collect();
        let order_str: Vec<&str> = order.iter().map(|s| s.as_str()).collect();
        assert!(
            order_str == expected || order_str == expected_rev,
            "OLO ytdist order {:?} does not match {:?} or {:?}",
            order_str,
            expected,
            expected_rev
        );
    }

    #[test]
    fn test_scipy_olo_x_ward() {
        // SciPy `linkage_X_ward_olo = [[4,3,...],[5,1,...],[2,0,...],
        // [6,8,...],[7,9,...]]` decodes to the tree
        // ((5,1),((4,3),(2,0))) with left-to-right leaves [5,1,4,3,2,0].
        // Pairwise Euclidean distances of X (6 points, condensed upper-triangle
        // order (0,1),(0,2),...,(4,5)), computed via `pdist(X)`.
        let names: Vec<String> = (0..6).map(|i| i.to_string()).collect();
        let condensed = vec![
            15.41753, 2.557604, 6.62464, 6.967422, 17.176438, // (0,*) pairs
            17.004015, 12.910675, 12.98128, 1.770454, // (1,*) pairs
            6.184128, 6.457123, 18.77446, // (2,*) pairs
            0.36266, 14.603109, // (3,*) pairs
            14.659662, // (4,5)
        ];
        let (mut tree, dist) =
            build_tree_for_olo(names, condensed, crate::libs::clust::hier::Method::Ward);

        optimal_leaf_order(&mut tree, &dist).unwrap();

        let order = leaf_name_order(&tree);
        let expected = vec!["5", "1", "4", "3", "2", "0"];
        let expected_rev: Vec<&str> = expected.iter().rev().copied().collect();
        let order_str: Vec<&str> = order.iter().map(|s| s.as_str()).collect();
        assert!(
            order_str == expected || order_str == expected_rev,
            "OLO X_ward order {:?} does not match {:?} or {:?}",
            order_str,
            expected,
            expected_rev
        );
    }
}
