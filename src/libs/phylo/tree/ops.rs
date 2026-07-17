use super::Tree;
use crate::libs::phylo::node::NodeId;
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// Check if `ancestor` is on the parent chain of `descendant`.
fn is_ancestor_of(tree: &Tree, ancestor: NodeId, descendant: NodeId) -> bool {
    let mut current = descendant;
    while let Some(node) = tree.get_node(current) {
        if let Some(parent_id) = node.parent {
            if parent_id == ancestor {
                return true;
            }
            current = parent_id;
        } else {
            break;
        }
    }
    false
}

/// Validate basic tree invariants after destructive operations.
///
/// Checks that:
/// * the root exists and is not deleted,
/// * every non-deleted node is reachable from the root,
/// * every non-root node has exactly one parent and appears in that parent's
///   children list,
/// * the root has no parent.
fn validate_tree_integrity(tree: &Tree) -> anyhow::Result<()> {
    let root = tree
        .get_root()
        .ok_or_else(|| anyhow::anyhow!("tree has no root"))?;
    if tree.get_node(root).is_none() {
        anyhow::bail!("root node {} is deleted", root);
    }

    // Reachability from root.
    let mut reachable = HashSet::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        if !reachable.insert(id) {
            continue;
        }
        let Some(node) = tree.get_node(id) else {
            anyhow::bail!("reachable node {} not found", id);
        };
        for &child in &node.children {
            if tree.get_node(child).is_none() {
                anyhow::bail!("node {} has deleted child {}", id, child);
            }
            stack.push(child);
        }
    }

    for (id, node) in tree.nodes.iter().enumerate() {
        if node.deleted {
            continue;
        }
        if !reachable.contains(&id) {
            anyhow::bail!("node {} is not reachable from root", id);
        }
        if id == root {
            if node.parent.is_some() {
                anyhow::bail!("root node {} has a parent", id);
            }
            continue;
        }
        let parent = node
            .parent
            .ok_or_else(|| anyhow::anyhow!("non-root node {} has no parent", id))?;
        let parent_node = tree
            .get_node(parent)
            .ok_or_else(|| anyhow::anyhow!("parent {} of node {} is deleted", parent, id))?;
        if !parent_node.children.contains(&id) {
            anyhow::bail!("node {} is not listed as a child of parent {}", id, parent);
        }
    }

    Ok(())
}

/// Add a child to a parent node.
/// Updates both parent's `children` list and child's `parent` field.
pub fn add_child(tree: &mut Tree, parent_id: NodeId, child_id: NodeId) -> anyhow::Result<()> {
    if parent_id == child_id {
        anyhow::bail!("Cannot add node as child of itself");
    }
    if tree.get_node(parent_id).is_none() {
        anyhow::bail!("Parent node {} not found or deleted", parent_id);
    }
    if tree.get_node(child_id).is_none() {
        anyhow::bail!("Child node {} not found or deleted", child_id);
    }

    // Check if child already has a parent
    let child_parent = {
        let child = tree
            .get_node(child_id)
            .ok_or_else(|| anyhow::anyhow!("Child node {} not found or deleted", child_id))?;
        child.parent
    };
    if let Some(old_parent) = child_parent {
        anyhow::bail!("Node {} already has parent {}", child_id, old_parent);
    }
    if is_ancestor_of(tree, child_id, parent_id) {
        anyhow::bail!(
            "Cannot add node {} as child of {}; it would create a cycle",
            child_id,
            parent_id
        );
    }

    if let Some(child) = tree.get_node_mut(child_id) {
        child.parent = Some(parent_id);
    }
    if let Some(parent) = tree.get_node_mut(parent_id) {
        parent.children.push(child_id);
    }

    Ok(())
}

/// Remove a node from its parent's children list, mark it as deleted, and
/// clear its parent/children pointers. Updates the tree root if necessary.
fn detach_and_delete(tree: &mut Tree, id: NodeId) {
    if tree.get_node(id).is_none() {
        return;
    }
    let parent_id = tree.get_node(id).and_then(|n| n.parent);
    if let Some(parent_id) = parent_id {
        if let Some(parent) = tree.get_node_mut(parent_id) {
            parent.children.retain(|&child| child != id);
        }
    }

    if let Some(node) = tree.get_node_mut(id) {
        node.deleted = true;
        node.children.clear();
        node.parent = None;
    }

    if tree.root == Some(id) {
        tree.root = None;
    }
}

/// Soft remove a node and its descendants (optional recursive).
/// If recursive is false, children are orphaned (parent set to None).
pub fn remove_node(tree: &mut Tree, id: NodeId, recursive: bool) {
    if tree.get_node(id).is_none() {
        return;
    }

    if recursive {
        // Collect all descendants using an explicit stack. `std::mem::take`
        // moves the children vector out in place, avoiding a clone of every
        // child list during recursive deletion.
        let mut to_remove = vec![id];
        let mut stack = vec![id];
        while let Some(cur) = stack.pop() {
            // Guard against malformed trees where a child ID is out of bounds or deleted.
            if tree.get_node(cur).is_none() {
                continue;
            }
            let children = if let Some(node) = tree.get_node_mut(cur) {
                std::mem::take(&mut node.children)
            } else {
                continue;
            };
            for child_id in children {
                stack.push(child_id);
                to_remove.push(child_id);
            }
        }
        for cur in to_remove {
            detach_and_delete(tree, cur);
        }
    } else {
        // Orphan direct children without deleting them.
        let children = if let Some(node) = tree.get_node_mut(id) {
            std::mem::take(&mut node.children)
        } else {
            return;
        };
        for child_id in children {
            if let Some(child) = tree.get_node_mut(child_id) {
                child.parent = None;
            }
        }
        detach_and_delete(tree, id);
    }
}

/// Collapse a node, removing it and connecting its children to its parent.
/// Edge lengths are summed (parent->node + node->child). Non-finite,
/// negative, and zero lengths are normalized to 0.0 before summing, and sums
/// that are not positive are stored as `None`.
pub fn collapse_node(tree: &mut Tree, id: NodeId) -> anyhow::Result<()> {
    if tree.get_node(id).is_none() {
        anyhow::bail!("Node {} not found", id);
    }
    if tree.root == Some(id) {
        anyhow::bail!("Cannot collapse root node");
    }

    // 1. Get info. `finite_length()` normalizes non-finite/negative/zero
    // lengths to 0.0, matching the global branch-length semantics. We use the
    // normalized value for both "has length" and "length value" decisions so
    // that a `Some(0.0)` parent edge is treated the same as an absent length.
    let (parent_id, parent_len) = {
        let node = tree
            .get_node(id)
            .ok_or_else(|| anyhow::anyhow!("Node {} not found or deleted", id))?;
        let parent_id = node
            .parent
            .ok_or_else(|| anyhow::anyhow!("Node {} has no parent", id))?;
        (parent_id, node.finite_length())
    };
    let children_info: Vec<(NodeId, f64)> = tree
        .get_node(id)
        .map(|node| {
            node.children
                .iter()
                .filter_map(|&c| tree.get_node(c).map(|child| (c, child.finite_length())))
                .collect()
        })
        .unwrap_or_default();

    // 2. Re-parent children
    let mut new_children_ids = Vec::new();
    for (child_id, child_len) in children_info {
        let new_edge = match (parent_len > 0.0, child_len > 0.0) {
            (true, true) => {
                let sum = parent_len + child_len;
                if sum > 0.0 {
                    Some(sum)
                } else {
                    None
                }
            }
            (true, false) => Some(parent_len),
            (false, true) => Some(child_len),
            (false, false) => None,
        };

        // Update child
        if let Some(child) = tree.get_node_mut(child_id) {
            child.parent = Some(parent_id);
            child.length = new_edge;
        }
        new_children_ids.push(child_id);
    }

    // 3. Update parent's children list
    if let Some(parent) = tree.get_node_mut(parent_id) {
        let pos = parent
            .children
            .iter()
            .position(|&x| x == id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "node {} is not recorded as a child of parent {}",
                    id,
                    parent_id
                )
            })?;
        parent.children.splice(pos..pos + 1, new_children_ids);
    }

    // 4. Detach and mark deleted
    detach_and_delete(tree, id);

    Ok(())
}

/// Compact the tree by removing soft-deleted nodes and remapping IDs.
/// This invalidates all existing NodeIds held outside!
pub fn compact(tree: &mut Tree) {
    let mut old_to_new = std::collections::HashMap::new();
    let mut new_nodes = Vec::with_capacity(tree.nodes.len());
    let mut new_idx = 0;

    // 1. Build mapping and new node list (without edges first)
    for old_node in &tree.nodes {
        if !old_node.deleted {
            old_to_new.insert(old_node.id, new_idx);
            // Create a shallow copy with updated ID but empty edges (will fill later)
            let mut new_node = old_node.clone();
            new_node.id = new_idx;
            new_node.parent = None; // Reset relations
            new_node.children.clear();
            new_nodes.push(new_node);
            new_idx += 1;
        }
    }

    // 2. Reconstruct edges using the mapping
    for (old_idx, node) in tree.nodes.iter().enumerate() {
        if node.deleted {
            continue;
        }

        let Some(&new_self_idx) = old_to_new.get(&old_idx) else {
            continue;
        };

        // Remap parent
        if let Some(old_parent) = node.parent {
            if let Some(&new_parent) = old_to_new.get(&old_parent) {
                new_nodes[new_self_idx].parent = Some(new_parent);
            }
        }

        // Remap children
        for &old_child in &node.children {
            if let Some(&new_child) = old_to_new.get(&old_child) {
                new_nodes[new_self_idx].children.push(new_child);
            }
        }
    }

    // 3. Update root
    if let Some(old_root) = tree.root {
        tree.root = old_to_new.get(&old_root).copied();
    }

    // 4. Swap
    tree.nodes = new_nodes;
}

/// Insert a node in the middle of the desired node and its parent.
/// Returns the new parent node ID.
pub fn insert_parent(tree: &mut Tree, id: NodeId) -> anyhow::Result<NodeId> {
    let node = tree
        .get_node(id)
        .ok_or_else(|| anyhow::anyhow!("Node {} not found", id))?;
    let parent = node
        .parent
        .ok_or_else(|| anyhow::anyhow!("Node {} has no parent", id))?;
    let length = node.length;
    let half_len = node.finite_length() / 2.0;
    let new_length = length.map(|_| half_len);

    let new_node = tree.add_node();

    // Link parent -> new_node
    add_child(tree, parent, new_node)?;
    if let Some(n) = tree.get_node_mut(new_node) {
        n.length = new_length;
    }

    // Unlink parent -> id
    if let Some(p_node) = tree.get_node_mut(parent) {
        p_node.children.retain(|&c| c != id);
    }
    // Update id parent
    if let Some(node) = tree.get_node_mut(id) {
        node.parent = None;
    }

    // Link new_node -> id
    add_child(tree, new_node, id)?;
    if let Some(node) = tree.get_node_mut(id) {
        node.length = new_length;
    }

    Ok(new_node)
}

/// Insert a new parent node above two sibling nodes.
/// Returns the new parent node ID.
///
/// # Errors
///
/// Returns an error if the nodes do not share the same parent, or if either
/// node is not found. Restricting this to siblings prevents creating cycles
/// when one node is an ancestor of the other.
pub fn insert_parent_pair(tree: &mut Tree, id1: NodeId, id2: NodeId) -> anyhow::Result<NodeId> {
    if id1 == id2 {
        anyhow::bail!("Cannot insert a common parent for the same node {}", id1);
    }

    let node1 = tree
        .get_node(id1)
        .ok_or_else(|| anyhow::anyhow!("Node {} not found", id1))?;
    let node2 = tree
        .get_node(id2)
        .ok_or_else(|| anyhow::anyhow!("Node {} not found", id2))?;

    let parent1 = node1
        .parent
        .ok_or_else(|| anyhow::anyhow!("Node {} has no parent", id1))?;
    let parent2 = node2
        .parent
        .ok_or_else(|| anyhow::anyhow!("Node {} has no parent", id2))?;

    if parent1 != parent2 {
        anyhow::bail!(
            "Nodes {} and {} are not siblings (parents {} and {} differ)",
            id1,
            id2,
            parent1,
            parent2
        );
    }

    let old = parent1;
    let edge1 = node1.length;
    let edge2 = node2.length;

    // New node with parent (old) has no edge length
    let new = tree.add_node();
    add_child(tree, old, new)?;

    // Move children to new node
    // 1. Unlink from their current parent
    if let Some(p_node) = tree.get_node_mut(old) {
        p_node.children.retain(|&c| c != id1 && c != id2);
    }

    if let Some(node) = tree.get_node_mut(id1) {
        node.parent = None;
    }
    if let Some(node) = tree.get_node_mut(id2) {
        node.parent = None;
    }

    // 2. Link to new
    add_child(tree, new, id1)?;
    if let Some(node) = tree.get_node_mut(id1) {
        node.length = edge1;
    }

    add_child(tree, new, id2)?;
    if let Some(node) = tree.get_node_mut(id2) {
        node.length = edge2;
    }

    Ok(new)
}

/// Remove nodes that have a parent and exactly one child (degree 2 nodes).
/// This is often used after rerooting to clean up the tree.
pub fn remove_degree_two_nodes(tree: &mut Tree) -> anyhow::Result<()> {
    // Collect all degree-2 nodes in one pass.
    // Collapsing a degree-2 node replaces it with its child in the parent's
    // children list, so the parent's degree is unchanged and no new degree-2
    // nodes are created.
    let to_remove: Vec<NodeId> = tree.find_nodes(|n| n.parent.is_some() && n.children.len() == 1);

    for id in to_remove {
        // Skip nodes that may have been altered by a previous collapse
        let need_remove = tree
            .get_node(id)
            .map(|n| n.parent.is_some() && n.children.len() == 1)
            .unwrap_or(false);
        if need_remove {
            collapse_node(tree, id)?;
        }
    }

    Ok(())
}

/// Deroot the tree by converting a bifurcating root into a multifurcating root.
///
/// Both children of the root are removed and their children are promoted to be
/// direct children of the root. Edge lengths from the root to each removed
/// child are added to that child's descendants, matching the behavior of
/// `collapse_node`. Non-finite, negative, and zero lengths are normalized to
/// 0.0 before summing, and non-positive sums are stored as `None`.
pub fn deroot(tree: &mut Tree) -> anyhow::Result<()> {
    let root = tree.root.ok_or_else(|| anyhow::anyhow!("Empty tree"))?;
    let children = tree
        .get_node(root)
        .ok_or_else(|| anyhow::anyhow!("root node {} not found", root))?
        .children
        .clone();

    if children.len() != 2 {
        anyhow::bail!("Root is not bifurcating (degree != 2)");
    }

    let mut new_children = Vec::new();
    let mut to_delete = Vec::new();

    for &child_id in &children {
        let is_leaf = tree
            .get_node(child_id)
            .map(|child| child.is_leaf())
            .unwrap_or(false);

        if is_leaf {
            // Leaf children remain direct children of the root.
            new_children.push(child_id);
        } else {
            to_delete.push(child_id);
            let (parent_len, grandchildren) = {
                let child = tree
                    .get_node(child_id)
                    .ok_or_else(|| anyhow::anyhow!("child node {} not found", child_id))?;
                (child.finite_length(), child.children.clone())
            };
            for &grandchild_id in &grandchildren {
                if let Some(grandchild) = tree.get_node_mut(grandchild_id) {
                    grandchild.parent = Some(root);
                    // `finite_length()` normalizes non-finite/negative/zero
                    // lengths to 0.0; non-positive sums become `None`.
                    let new_len = parent_len + grandchild.finite_length();
                    grandchild.length = if new_len > 0.0 { Some(new_len) } else { None };
                }
                new_children.push(grandchild_id);
            }
        }
    }

    if let Some(root_node) = tree.get_node_mut(root) {
        root_node.children = new_children;
    }

    for id in to_delete {
        detach_and_delete(tree, id);
    }

    Ok(())
}

/// Reroot the tree at the specified node.
/// This reverses the direction of edges along the path from the old root to the new root.
pub fn reroot_at(
    tree: &mut Tree,
    new_root_id: NodeId,
    process_support_values: bool,
) -> anyhow::Result<()> {
    if tree.get_node(new_root_id).is_none() {
        anyhow::bail!("Node {} not found", new_root_id);
    }

    let old_root_id = tree
        .root
        .ok_or_else(|| anyhow::anyhow!("Tree has no root"))?;
    if old_root_id == new_root_id {
        return Ok(());
    }

    // 1. Get path from old root to new root
    let path = tree.get_path_from_root(new_root_id)?;

    // 1.5 Process Support Values (Labels)
    // Shift internal node labels along the path to align with edge reversals.
    // Internal node names are treated as support values for the edge immediately
    // above them (the edge connecting the node to its parent). When rerooting,
    // edges along the path are reversed, so each support label must follow the
    // edge it annotates. Leaves keep their taxon names and are never modified.
    if process_support_values {
        let new_root_is_leaf = tree
            .get_node(new_root_id)
            .map(|n| n.children.is_empty())
            .unwrap_or(false);

        // Capture original names
        let mut names = Vec::with_capacity(path.len());
        for &id in &path {
            let node = tree
                .get_node(id)
                .ok_or_else(|| anyhow::anyhow!("path node {} not found", id))?;
            names.push(node.name.clone());
        }

        // Each internal node's name annotates the edge to its parent. After
        // reversing the edges on the path, the label must move to the node
        // that now sits on that edge. The old root absorbs the label from the
        // node directly below it; leaves keep their taxon names.
        for i in 0..path.len() {
            let node_id = path[i];
            // Only modify internal nodes (leaves keep Taxon names)
            // Note: All nodes on path except possibly the last one are ancestors, thus internal.
            let is_leaf = (i == path.len() - 1) && new_root_is_leaf;

            if !is_leaf {
                let new_name = if i < path.len() - 1 {
                    // Take from next node, UNLESS next node is a leaf (Taxon)
                    let next_is_leaf = (i + 1 == path.len() - 1) && new_root_is_leaf;
                    if next_is_leaf {
                        None
                    } else {
                        names[i + 1].clone()
                    }
                } else {
                    // New root (internal): takes label from old root
                    names[0].clone()
                };

                if let Some(node) = tree.get_node_mut(node_id) {
                    node.name = new_name;
                }
            }
        }
    }

    // 2. Collect edge lengths along the path
    // path[i]'s length represents edge (path[i-1] -> path[i])
    let mut lengths = Vec::with_capacity(path.len());
    for &id in &path {
        let node = tree
            .get_node(id)
            .ok_or_else(|| anyhow::anyhow!("path node {} not found", id))?;
        lengths.push(node.length);
    }

    // 3. Reverse edges
    for i in (1..path.len()).rev() {
        let child_id = path[i];
        let parent_id = path[i - 1];
        let length = lengths[i];

        // Defensive check: detect malformed input that would introduce a cycle.
        if tree
            .get_node(child_id)
            .map(|n| n.children.contains(&parent_id))
            .unwrap_or(false)
        {
            anyhow::bail!(
                "reroot would create a cycle between nodes {} and {}",
                parent_id,
                child_id
            );
        }

        // a. Remove child from parent's children
        if let Some(parent) = tree.get_node_mut(parent_id) {
            parent.children.retain(|&x| x != child_id);
        }

        // b. Add parent to child's children
        if let Some(child) = tree.get_node_mut(child_id) {
            child.children.push(parent_id);
        }

        // c. Update parent's parent pointer and length
        if let Some(parent) = tree.get_node_mut(parent_id) {
            parent.parent = Some(child_id);
            parent.length = length;
        }
    }

    // 4. Finalize new root
    if let Some(new_root) = tree.get_node_mut(new_root_id) {
        new_root.parent = None;
        new_root.length = None;
    }

    tree.root = Some(new_root_id);

    // Defensive: ensure the rerooted tree is still a valid rooted tree.
    validate_tree_integrity(tree)
}

/// Prune nodes that match a predicate.
/// Warning: This removes the matching nodes AND their descendants.
pub fn prune_where<F>(tree: &mut Tree, predicate: F)
where
    F: Fn(&crate::libs::phylo::node::Node) -> bool,
{
    // We need to collect IDs first to avoid borrowing issues
    let to_remove: Vec<NodeId> = tree
        .nodes
        .iter()
        .filter(|n| !n.deleted && predicate(n))
        .map(|n| n.id)
        .collect();

    for id in to_remove {
        remove_node(tree, id, true);
    }
}

/// Condense the subtree rooted at `sub_root_id` into a single node with the given name.
/// The new node inherits the edge length of the subtree root and gets `member` and `tri=white` properties.
pub fn condense_subtree(
    tree: &mut Tree,
    sub_root_id: NodeId,
    name: &str,
    member_count: usize,
) -> anyhow::Result<()> {
    let sub_root = tree
        .get_node(sub_root_id)
        .ok_or_else(|| anyhow::anyhow!("Node {} not found", sub_root_id))?;
    let parent_id_opt = sub_root.parent;
    let edge_len = sub_root.length;

    if let Some(parent_id) = parent_id_opt {
        // Subtree root has a parent: replace subtree with a single child node.
        let new_node_id = tree.add_node();
        if let Some(node) = tree.get_node_mut(new_node_id) {
            node.set_name(name);
            node.length = edge_len;
            node.add_property("member", member_count.to_string());
            node.add_property("tri", "white");
        }

        tree.remove_node(sub_root_id, true);
        tree.add_child(parent_id, new_node_id)?;
    } else {
        // Subtree root is the tree root: replace entire tree with a single root node.
        tree.remove_node(sub_root_id, true);

        let new_root = tree.add_node();
        tree.set_root(new_root)?;
        if let Some(node) = tree.get_node_mut(new_root) {
            node.set_name(name);
            node.add_property("member", member_count.to_string());
            node.add_property("tri", "white");
        }
    }

    Ok(())
}

/// Remove node properties matching a regex.
///
/// For every node in `tree`, each property entry is serialized as `key=value`
/// (or just `key` when the value is empty) and tested against `pattern`
/// (case-insensitive, ASCII-only). Matching entries are removed in place.
pub fn remove_properties_matching(tree: &mut Tree, pattern: &str) -> anyhow::Result<()> {
    let re = regex::RegexBuilder::new(pattern)
        .case_insensitive(true)
        .unicode(false)
        .build()?;

    for i in 0..tree.nodes.len() {
        if let Some(node) = tree.get_node_mut(i) {
            if let Some(props) = &mut node.properties {
                let mut to_remove = Vec::new();
                for (k, v) in props.iter() {
                    let entry = if v.is_empty() {
                        k.to_string()
                    } else {
                        format!("{}={}", k, v)
                    };
                    if re.is_match(&entry) {
                        to_remove.push(k.clone());
                    }
                }
                for k in to_remove {
                    props.remove(&k);
                }
            }
        }
    }

    Ok(())
}

/// Mode for replacing node annotations in `replace_annotations`.
#[derive(Debug, Clone, Copy)]
pub enum AnnotationMode {
    /// Replace the node name.
    Label,
    /// Add an NCBI TaxID property (`:T=`).
    TaxId,
    /// Add a species name property (`:S=`).
    Species,
    /// Append property/comment as-is.
    AsIs,
}

/// Append a single property entry parsed as `key=value` or a bare key.
fn append_property_item(node: &mut crate::libs::phylo::node::Node, item: &str) {
    if let Some((key, value)) = item.split_once('=') {
        node.add_property(key, value);
    } else {
        node.add_property(item, "");
    }
}

/// Append remaining non-empty replacement values as property entries.
fn append_remaining_properties<'a>(
    node: &mut crate::libs::phylo::node::Node,
    values: impl Iterator<Item = &'a String>,
) {
    for item in values.filter(|s| !s.is_empty()) {
        append_property_item(node, item);
    }
}

/// Replace node names or append NHX-style annotations from a mapping.
pub fn replace_annotations(
    tree: &mut Tree,
    mode: AnnotationMode,
    mapping: &BTreeMap<String, Vec<String>>,
    skip_internal: bool,
    skip_leaf: bool,
) -> anyhow::Result<()> {
    let duplicates = super::stat::duplicate_names(tree);
    for (original, replacements) in mapping {
        let Some(id) = tree.get_node_by_name(original) else {
            continue;
        };
        if duplicates.contains(original) {
            log::warn!(
                "duplicate node name '{}' matched multiple nodes; replacing only the first match",
                original
            );
        }
        let is_leaf = tree.get_node(id).map(|n| n.is_leaf()).unwrap_or(true);
        if skip_internal && !is_leaf {
            continue;
        }
        if skip_leaf && is_leaf {
            continue;
        }
        let Some(node) = tree.get_node_mut(id) else {
            continue;
        };

        let mut values = replacements.iter();
        match mode {
            AnnotationMode::Label => {
                if let Some(first) = values.next() {
                    if first.is_empty() {
                        node.name = None;
                    } else {
                        node.set_name(first);
                    }
                }
                append_remaining_properties(node, values);
            }
            AnnotationMode::TaxId => {
                if let Some(first) = values.next().filter(|s| !s.is_empty()) {
                    node.add_property("T", first);
                }
                append_remaining_properties(node, values);
            }
            AnnotationMode::Species => {
                if let Some(first) = values.next().filter(|s| !s.is_empty()) {
                    node.add_property("S", first);
                }
                append_remaining_properties(node, values);
            }
            AnnotationMode::AsIs => {
                append_remaining_properties(node, values);
            }
        }
    }
    Ok(())
}

/// Strip branch lengths, comments, and/or labels according to flags.
pub fn strip_topology(
    tree: &mut Tree,
    keep_length: bool,
    keep_comment: bool,
    remove_internal_labels: bool,
    remove_leaf_labels: bool,
) {
    let ids: Vec<NodeId> = tree
        .nodes
        .iter()
        .filter(|n| !n.deleted)
        .map(|n| n.id)
        .collect();
    for id in ids {
        if let Some(node) = tree.get_node_mut(id) {
            if !keep_length {
                node.length = None;
            }
            if !keep_comment {
                node.properties = None;
            }
            if node.is_leaf() && remove_leaf_labels {
                node.name = None;
            }
            if !node.is_leaf() && remove_internal_labels {
                node.name = None;
            }
        }
    }
}

/// Reroot the tree on the edge above the LCA of `target_ids`.
///
/// When `lax` is true and the LCA equals the current root, falls back to the
/// complement LCA (unspecified leaves). Returns `Ok(())` with the tree
/// unchanged when the LCA is already the root.
pub fn reroot_at_lca(
    tree: &mut Tree,
    target_ids: &BTreeSet<NodeId>,
    lax: bool,
    process_support: bool,
) -> anyhow::Result<()> {
    if target_ids.is_empty() {
        return Ok(());
    }

    let nodes: Vec<NodeId> = target_ids.iter().cloned().collect();
    let mut sub_root_id = tree.get_lca(&nodes)?;

    let old_root = tree
        .get_root()
        .ok_or_else(|| anyhow::anyhow!("tree has no root"))?;

    if old_root == sub_root_id && lax {
        if let Some(comp_lca) =
            crate::libs::phylo::tree::query::lax_complement_lca(tree, target_ids, old_root)
        {
            sub_root_id = comp_lca;
        }
    }

    if old_root == sub_root_id {
        return Ok(());
    }

    let new_root = tree.insert_parent(sub_root_id)?;
    tree.reroot_at(new_root, process_support)?;
    tree.remove_degree_two_nodes()?;

    Ok(())
}

/// Reroot at the midpoint of the longest branch. No-op when the tree has no
/// edges.
pub fn reroot_at_longest_branch(tree: &mut Tree, process_support: bool) -> anyhow::Result<()> {
    // Skip when no meaningful branch lengths exist (cladogram); otherwise
    // get_node_with_longest_edge would return an arbitrary node due to
    // tie-breaking, producing a meaningless reroot.
    let has_length = tree
        .nodes
        .iter()
        .any(|n| !n.deleted && n.length.map(|l| l > 0.0).unwrap_or(false));
    if !has_length {
        log::debug!("reroot_at_longest_branch: tree has no positive branch lengths, skipping");
        return Ok(());
    }

    if let Some(longest_node) = tree.get_node_with_longest_edge() {
        let new_root = tree.insert_parent(longest_node)?;
        tree.reroot_at(new_root, process_support)?;
        tree.remove_degree_two_nodes()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::phylo::tree::Tree;

    #[test]
    fn collapse_node_sums_finite_lengths() {
        let mut tree = Tree::from_newick("(A:0.1,(B:0.2,C:0.3)D:0.4)Root;").unwrap();
        let d_id = tree.get_node_by_name("D").unwrap();
        collapse_node(&mut tree, d_id).unwrap();

        let root = tree.get_root().unwrap();
        let children: Vec<NodeId> = tree.get_node(root).unwrap().children.clone();
        assert_eq!(children.len(), 3);
        let child_lengths: Vec<f64> = children
            .iter()
            .map(|&id| tree.get_node(id).unwrap().finite_length())
            .collect();
        assert!(child_lengths.contains(&0.1));
        assert!(child_lengths.contains(&(0.2 + 0.4)));
        assert!(child_lengths.contains(&(0.3 + 0.4)));
    }

    #[test]
    fn collapse_node_zero_parent_length_uses_child_lengths() {
        let mut tree = Tree::from_newick("(A:0.1,(B:0.2,C:0.3)D:0.0)Root;").unwrap();
        let d_id = tree.get_node_by_name("D").unwrap();
        collapse_node(&mut tree, d_id).unwrap();

        let root = tree.get_root().unwrap();
        let child_lengths: Vec<f64> = tree
            .get_node(root)
            .unwrap()
            .children
            .iter()
            .map(|&id| tree.get_node(id).unwrap().finite_length())
            .collect();
        assert!(child_lengths.contains(&0.1));
        assert!(child_lengths.contains(&0.2));
        assert!(child_lengths.contains(&0.3));
    }

    #[test]
    fn collapse_node_nan_child_length_treated_as_zero() {
        let mut tree = Tree::from_newick("(A:0.1,(B:0.2,C:0.3)D:0.4)Root;").unwrap();
        let b_id = tree.get_node_by_name("B").unwrap();
        tree.get_node_mut(b_id).unwrap().length = Some(f64::NAN);
        let d_id = tree.get_node_by_name("D").unwrap();
        collapse_node(&mut tree, d_id).unwrap();

        // NaN child length is normalized to 0.0, so B gets only the parent length.
        let b_new_len = tree.get_node(b_id).unwrap().finite_length();
        assert!((b_new_len - 0.4).abs() < 1e-9);
        let c_new_len = tree.get_node_by_name("C").unwrap();
        let c_new_len = tree.get_node(c_new_len).unwrap().finite_length();
        assert!((c_new_len - 0.7).abs() < 1e-9);
    }

    #[test]
    fn replace_annotations_handles_duplicate_names() {
        // Two nodes named A; replace_annotations should match the first one
        // and not panic. The visible result is that one A is renamed.
        let mut tree = Tree::from_newick("((A,A),B);").unwrap();
        let mut mapping = BTreeMap::new();
        mapping.insert("A".to_string(), vec!["X".to_string()]);
        replace_annotations(&mut tree, AnnotationMode::Label, &mapping, false, false).unwrap();
        let names: Vec<_> = tree.get_names();
        assert!(names.contains(&"X".to_string()));
        assert!(names.contains(&"A".to_string()));
        assert!(names.contains(&"B".to_string()));
    }

    #[test]
    fn reroot_at_rejects_malformed_tree() {
        // Manually corrupt a tree so a node has the wrong parent pointer.
        // reroot_at should detect the invariant violation and return an error.
        let mut tree = Tree::from_newick("((A,B),(C,D));").unwrap();
        let a_id = tree.get_node_by_name("A").unwrap();
        // Make A its own parent to break the tree.
        tree.get_node_mut(a_id).unwrap().parent = Some(a_id);
        assert!(reroot_at(&mut tree, a_id, false).is_err());
    }

    #[test]
    fn reroot_at_to_child_shifts_support_labels() {
        // Original tree: root 90 with two internal children 70 and 80.
        // Reroot at the internal node labeled 70.
        // The old root label (90) moves to the new root; the node that used
        // to be 70 now sits below the new root and takes label 70.
        let mut tree = Tree::from_newick("((A,B)70,(C,D)80)90;").unwrap();
        let target = tree.get_node_by_name("70").unwrap();
        reroot_at(&mut tree, target, true).unwrap();

        assert_eq!(tree.to_newick(), "(A,B,((C,D)80)70)90;");
        let names: std::collections::HashSet<_> = tree.get_names().into_iter().collect();
        assert!(names.contains("90"));
        assert!(names.contains("80"));
        assert!(names.contains("70"));
    }

    #[test]
    fn reroot_at_to_leaf_preserves_leaf_name_and_clears_path_internal() {
        // Reroot at leaf C. The internal node directly above C (80) is
        // converted into the new root's child and must lose its support label
        // so that leaf C keeps its taxon name.
        let mut tree = Tree::from_newick("((A,B)70,(C,D)80)90;").unwrap();
        let target = tree.get_node_by_name("C").unwrap();
        reroot_at(&mut tree, target, true).unwrap();

        assert_eq!(tree.to_newick(), "((D,((A,B)70)80))C;");
        let names: std::collections::HashSet<_> = tree.get_names().into_iter().collect();
        assert!(names.contains("C"));
        assert!(names.contains("70"));
        assert!(names.contains("80"));
        // The node directly above C should no longer have a support label.
        assert!(!names.contains("90"));
    }

    #[test]
    fn reroot_at_support_values_do_not_overwrite_other_internals() {
        // Reroot at the right internal node. The untouched left internal node
        // keeps its original support label.
        let mut tree = Tree::from_newick("((A,B)70,(C,D)80)90;").unwrap();
        let target = tree.get_node_by_name("80").unwrap();
        reroot_at(&mut tree, target, true).unwrap();

        let names: std::collections::HashSet<_> = tree.get_names().into_iter().collect();
        assert!(
            names.contains("70"),
            "untouched internal label should be preserved"
        );
        assert!(names.contains("80"));
        assert!(names.contains("90"));
    }
}
