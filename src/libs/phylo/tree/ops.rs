use super::Tree;
use crate::libs::phylo::node::NodeId;
use std::collections::BTreeSet;

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
/// Edge lengths are summed (parent->node + node->child).
pub fn collapse_node(tree: &mut Tree, id: NodeId) -> anyhow::Result<()> {
    if tree.get_node(id).is_none() {
        anyhow::bail!("Node {} not found", id);
    }
    if tree.root == Some(id) {
        anyhow::bail!("Cannot collapse root node");
    }

    // 1. Get info
    let (parent_id, parent_has_len, parent_len) = {
        let node = tree
            .get_node(id)
            .ok_or_else(|| anyhow::anyhow!("Node {} not found or deleted", id))?;
        let parent_id = node
            .parent
            .ok_or_else(|| anyhow::anyhow!("Node {} has no parent", id))?;
        (parent_id, node.length.is_some(), node.finite_length())
    };
    let children_info: Vec<(NodeId, bool, f64)> = tree
        .get_node(id)
        .map(|node| {
            node.children
                .iter()
                .filter_map(|&c| {
                    tree.get_node(c)
                        .map(|child| (c, child.length.is_some(), child.finite_length()))
                })
                .collect()
        })
        .unwrap_or_default();

    // 2. Re-parent children
    let mut new_children_ids = Vec::new();
    for (child_id, child_has_len, child_len) in children_info {
        let new_edge = match (parent_has_len, child_has_len) {
            (true, true) => {
                let sum = parent_len + child_len;
                if sum > 0.0 {
                    Some(sum)
                } else {
                    None
                }
            }
            (true, false) if parent_len > 0.0 => Some(parent_len),
            (false, true) if child_len > 0.0 => Some(child_len),
            _ => None,
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
/// `collapse_node`.
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

    Ok(())
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

/// Replace node names or append NHX-style annotations from a mapping.
pub fn replace_annotations(
    tree: &mut Tree,
    mode: AnnotationMode,
    mapping: &[(String, Vec<String>)],
    skip_internal: bool,
    skip_leaf: bool,
) -> anyhow::Result<()> {
    for (original, replacements) in mapping {
        let Some(id) = tree.get_node_by_name(original) else {
            continue;
        };
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
                for item in values.filter(|s| !s.is_empty()) {
                    if let Some((key, value)) = item.split_once('=') {
                        node.add_property(key, value);
                    } else {
                        node.add_property(item, "");
                    }
                }
            }
            AnnotationMode::TaxId => {
                if let Some(first) = values.next().filter(|s| !s.is_empty()) {
                    node.add_property("T", first);
                }
                for item in values.filter(|s| !s.is_empty()) {
                    if let Some((key, value)) = item.split_once('=') {
                        node.add_property(key, value);
                    } else {
                        node.add_property(item, "");
                    }
                }
            }
            AnnotationMode::Species => {
                if let Some(first) = values.next().filter(|s| !s.is_empty()) {
                    node.add_property("S", first);
                }
                for item in values.filter(|s| !s.is_empty()) {
                    if let Some((key, value)) = item.split_once('=') {
                        node.add_property(key, value);
                    } else {
                        node.add_property(item, "");
                    }
                }
            }
            AnnotationMode::AsIs => {
                for item in values.filter(|s| !s.is_empty()) {
                    if let Some((key, value)) = item.split_once('=') {
                        node.add_property(key, value);
                    } else {
                        node.add_property(item, "");
                    }
                }
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
