use super::*;
use crate::libs::phylo::TreeComparison;
use std::collections::BTreeSet;

#[test]
fn test_tree_traversals() {
    let mut tree = Tree::new();
    //    0
    //   / \
    //  1   2
    // / \   \
    //3   4   5
    let n0 = tree.add_node();
    let n1 = tree.add_node();
    let n2 = tree.add_node();
    let n3 = tree.add_node();
    let n4 = tree.add_node();
    let n5 = tree.add_node();

    let _ = tree.set_root(n0);
    tree.add_child(n0, n1).unwrap();
    tree.add_child(n0, n2).unwrap();
    tree.add_child(n1, n3).unwrap();
    tree.add_child(n1, n4).unwrap();
    tree.add_child(n2, n5).unwrap();

    // Preorder: 0, 1, 3, 4, 2, 5
    let pre = tree.preorder(n0);
    assert_eq!(pre, vec![n0, n1, n3, n4, n2, n5]);

    // Postorder: 3, 4, 1, 5, 2, 0
    let post = tree.postorder(n0);
    assert_eq!(post, vec![n3, n4, n1, n5, n2, n0]);

    // Levelorder: 0, 1, 2, 3, 4, 5
    let level = tree.levelorder(n0);
    assert_eq!(level, vec![n0, n1, n2, n3, n4, n5]);
}

#[test]
fn test_tree_basic_ops() {
    let mut tree = Tree::new();

    // Create nodes
    // 0(root) -> 1, 2
    // 1 -> 3
    let n0 = tree.add_node();
    let n1 = tree.add_node();
    let n2 = tree.add_node();
    let n3 = tree.add_node();

    let _ = tree.set_root(n0);

    tree.add_child(n0, n1).unwrap();
    tree.add_child(n0, n2).unwrap();
    tree.add_child(n1, n3).unwrap();

    assert_eq!(tree.len(), 4);

    // Check structure
    let root = tree.get_node(n0).unwrap();
    assert_eq!(root.children, vec![n1, n2]);

    let node1 = tree.get_node(n1).unwrap();
    assert_eq!(node1.parent, Some(n0));
    assert_eq!(node1.children, vec![n3]);
}

#[test]
fn test_tree_remove_and_compact() {
    let mut tree = Tree::new();
    // 0 -> 1 -> 2
    let n0 = tree.add_node();
    let n1 = tree.add_node();
    let n2 = tree.add_node();

    tree.add_child(n0, n1).unwrap();
    tree.add_child(n1, n2).unwrap();
    let _ = tree.set_root(n0);

    // Remove n1 (recursive=false), n2 becomes orphan
    tree.remove_node(n1, false);

    assert!(tree.get_node(n1).is_none()); // n1 is logically gone
    assert_eq!(tree.len(), 2); // 0 and 2 remain

    let node0 = tree.get_node(n0).unwrap();
    assert!(!node0.children.contains(&n1)); // 0 no longer points to 1

    let node2 = tree.get_node(n2).unwrap();
    assert_eq!(node2.parent, None); // 2 is orphaned

    // Compact
    // Before: [0:Valid, 1:Deleted, 2:Valid]
    // After:  [0':Old0, 1':Old2]
    tree.compact();

    assert_eq!(tree.len(), 2);
    // Old n0 should be at index 0
    let new_n0 = tree.get_node(0).unwrap();
    assert_eq!(new_n0.children.len(), 0);

    // Old n2 should be at index 1
    let new_n1 = tree.get_node(1).unwrap();
    assert_eq!(new_n1.parent, None);
}

#[test]
fn test_tree_paths_and_distances() {
    let mut tree = Tree::new();
    //    0
    //   / \
    //  1   2
    // / \
    //3   4
    let n0 = tree.add_node();
    let n1 = tree.add_node();
    let n2 = tree.add_node();
    let n3 = tree.add_node();
    let n4 = tree.add_node();

    let _ = tree.set_root(n0);
    tree.add_child(n0, n1).unwrap();
    tree.add_child(n0, n2).unwrap();
    tree.add_child(n1, n3).unwrap();
    tree.add_child(n1, n4).unwrap();

    // Set lengths
    tree.get_node_mut(n1).unwrap().length = Some(1.0);
    tree.get_node_mut(n2).unwrap().length = Some(2.0);
    tree.get_node_mut(n3).unwrap().length = Some(3.0);
    tree.get_node_mut(n4).unwrap().length = Some(4.0);

    // Paths
    assert_eq!(tree.get_path_from_root(n3).unwrap(), vec![n0, n1, n3]);
    assert_eq!(tree.get_path_from_root(n2).unwrap(), vec![n0, n2]);

    // LCA
    assert_eq!(tree.get_common_ancestor(n3, n4).unwrap(), n1);
    assert_eq!(tree.get_common_ancestor(n3, n2).unwrap(), n0);
    assert_eq!(tree.get_common_ancestor(n1, n3).unwrap(), n1);

    // Distance
    // n3 -> n4: n3(3.0)->n1 + n1->n4(4.0) = 7.0 (weighted). Steps: n3->n1->n4 = 2 edges.
    let (w, t) = tree.get_distance(n3, n4).unwrap();
    assert_eq!(w, 7.0);
    assert_eq!(t, 2);

    // n3 -> n2: n3(3.0)->n1(1.0)->n0 + n0->n2(2.0) = 6.0. Steps: n3->n1->n0->n2 = 3 edges.
    let (w, t) = tree.get_distance(n3, n2).unwrap();
    assert_eq!(w, 6.0);
    assert_eq!(t, 3);
}

#[test]
fn node_distance_uses_branch_lengths_when_present() {
    // When the tree has branch lengths, node_distance returns the weighted sum.
    let tree = Tree::from_newick("(A:1e-10,B:1e-10)R;").unwrap();
    let a = tree.get_node_by_name("A").unwrap();
    let b = tree.get_node_by_name("B").unwrap();
    assert!((tree.node_distance(a, b).unwrap() - 2e-10).abs() < 1e-15);

    let tree = Tree::from_newick("(A:1.0,B:1.0)R;").unwrap();
    let a = tree.get_node_by_name("A").unwrap();
    let b = tree.get_node_by_name("B").unwrap();
    assert_eq!(tree.node_distance(a, b).unwrap(), 2.0);
}

#[test]
fn node_distance_falls_back_to_edge_count_for_cladogram() {
    // When the tree has no branch lengths, node_distance returns edge count.
    let tree = Tree::from_newick("(A,B)R;").unwrap();
    let a = tree.get_node_by_name("A").unwrap();
    let b = tree.get_node_by_name("B").unwrap();
    assert_eq!(tree.node_distance(a, b).unwrap(), 2.0);
}

#[test]
fn test_tree_queries() {
    let mut tree = Tree::new();
    //    0 (root, name="root")
    //   / \
    //  1   2 (leaf, name="leaf2")
    // /
    //3 (leaf, name="leaf3")
    let n0 = tree.add_node();
    tree.get_node_mut(n0).unwrap().name = Some("root".to_string());

    let n1 = tree.add_node();

    let n2 = tree.add_node();
    tree.get_node_mut(n2).unwrap().name = Some("leaf2".to_string());

    let n3 = tree.add_node();
    tree.get_node_mut(n3).unwrap().name = Some("leaf3".to_string());

    let _ = tree.set_root(n0);
    tree.add_child(n0, n1).unwrap();
    tree.add_child(n0, n2).unwrap();
    tree.add_child(n1, n3).unwrap();

    // Subtree
    // subtree(1) = [1, 3]
    let sub = tree.extract_subtree(n1).unwrap();
    // IDs might change, but names should persist?
    // Subtree renumbers. Root of subtree is 0. Child is 1.
    // Names: None, leaf3.
    let sub_root = sub.get_root().unwrap();
    assert_eq!(sub.get_node(sub_root).unwrap().name, None); // Old n1
    let children = &sub.get_node(sub_root).unwrap().children;
    assert_eq!(children.len(), 1);
    assert_eq!(
        sub.get_node(children[0]).unwrap().name,
        Some("leaf3".to_string())
    );

    // Leaves
    // Leaves: 2, 3
    let leaves = tree.get_leaves();
    assert!(leaves.contains(&n2));
    assert!(leaves.contains(&n3));
    assert_eq!(leaves.len(), 2);

    // Find nodes
    let named_nodes = tree.find_nodes(|n| n.name.is_some());
    assert_eq!(named_nodes.len(), 3); // 0, 2, 3

    // Get by name
    assert_eq!(tree.get_node_by_name("root"), Some(n0));
    assert_eq!(tree.get_node_by_name("leaf2"), Some(n2));
    assert_eq!(tree.get_node_by_name("leaf3"), Some(n3));
    assert_eq!(tree.get_node_by_name("nonexistent"), None);
}

#[test]
fn test_tree_prune() {
    let mut tree = Tree::new();
    //    0
    //   / \
    //  1   2
    // /
    //3
    let n0 = tree.add_node();
    tree.get_node_mut(n0).unwrap().name = Some("root".to_string());

    let n1 = tree.add_node();
    tree.get_node_mut(n1).unwrap().name = Some("n1".to_string());

    let n2 = tree.add_node();
    tree.get_node_mut(n2).unwrap().name = Some("remove_me".to_string());

    let n3 = tree.add_node();
    tree.get_node_mut(n3).unwrap().name = Some("n3".to_string());

    let _ = tree.set_root(n0);
    tree.add_child(n0, n1).unwrap();
    tree.add_child(n0, n2).unwrap();
    tree.add_child(n1, n3).unwrap();

    assert_eq!(tree.len(), 4);

    // Prune node with name "remove_me"
    tree.prune_where(|n| n.name.as_deref() == Some("remove_me"));

    assert_eq!(tree.len(), 3);
    assert!(tree.get_node(n2).is_none());
    assert!(tree.get_node(n0).unwrap().children.contains(&n1));
    assert!(!tree.get_node(n0).unwrap().children.contains(&n2));

    // Prune n1, should also remove n3
    tree.prune_where(|n| n.id == n1);

    assert_eq!(tree.len(), 1); // Only root left
    assert!(tree.get_node(n1).is_none());
    assert!(tree.get_node(n3).is_none());
    assert!(tree.get_node(n0).unwrap().children.is_empty());
}

#[test]
fn test_is_binary() {
    let t1 = Tree::from_newick("((A,B),C);").unwrap();
    assert!(t1.is_binary());

    let t2 = Tree::from_newick("(A,B,C);").unwrap();
    assert!(!t2.is_binary());
}

#[test]
fn test_get_leaves() {
    let tree = Tree::from_newick("((A,B)C,D)E;").unwrap();
    let leaves = tree.get_leaf_names();
    let leaf_names: Vec<String> = leaves.into_iter().map(|n| n.unwrap()).collect();

    assert!(leaf_names.contains(&"A".to_string()));
    assert!(leaf_names.contains(&"B".to_string()));
    assert!(leaf_names.contains(&"D".to_string()));
    assert_eq!(leaf_names.len(), 3);
}

#[test]
fn test_diameter() {
    let newick = "((A:1,B:2):1,C:4);";
    let tree = Tree::from_newick(newick).unwrap();
    // Dist(A,B) = 3
    // Dist(A,C) = 1+1+4 = 6
    // Dist(B,C) = 2+1+4 = 7
    assert_eq!(tree.diameter(), 7.0);
}

#[test]
fn test_robinson_foulds() {
    // 3 leaves -> Unrooted RF is always 0 (star topology)
    let t1 = Tree::from_newick("((A,B),C);").unwrap();
    let t2 = Tree::from_newick("((A,C),B);").unwrap();
    assert_eq!(t1.robinson_foulds(&t2).unwrap(), 0);

    let t3 = Tree::from_newick("((A,B),C);").unwrap();
    assert_eq!(t1.robinson_foulds(&t3).unwrap(), 0);

    // 4 leaves -> Unrooted RF can be non-zero
    let t4 = Tree::from_newick("((A,B),(C,D));").unwrap();
    let t5 = Tree::from_newick("((A,C),(B,D));").unwrap();
    assert_eq!(t4.robinson_foulds(&t5).unwrap(), 2);
}

#[test]
fn test_deroot() {
    let mut tree = Tree::from_newick("((A:1,B:2)C:3,D:4)Root;").unwrap();
    tree.deroot().unwrap();

    let root = tree.get_root().unwrap();
    let children = &tree.get_node(root).unwrap().children;
    assert_eq!(children.len(), 3);

    // Check names of children
    let child_names: Vec<String> = children
        .iter()
        .map(|&id| tree.get_node(id).unwrap().name.clone().unwrap_or_default())
        .collect();

    assert!(child_names.contains(&"A".to_string()));
    assert!(child_names.contains(&"B".to_string()));
    assert!(child_names.contains(&"D".to_string()));
}

#[test]
fn test_reroot_support_values() {
    let mut tree = Tree::from_newick("(A,(B,C)Support)Root;").unwrap();
    let c_id = tree.get_node_by_name("C").unwrap();

    tree.reroot_at(c_id, true).unwrap();

    // C is root
    assert_eq!(tree.get_root(), Some(c_id));

    let root = tree.get_node(tree.get_root().unwrap()).unwrap();
    assert_eq!(root.name.as_deref(), Some("C"));

    let support_node_id = root.children[0]; // The old Support node
    let support_node = tree.get_node(support_node_id).unwrap();
    assert_eq!(support_node.name, None);

    let old_root_node_id = support_node
        .children
        .iter()
        .find(|&&id| {
            // Find the one that has A as child
            let n = tree.get_node(id).unwrap();
            n.children
                .iter()
                .any(|&child| tree.get_node(child).unwrap().name.as_deref() == Some("A"))
        })
        .unwrap();

    let old_root_node = tree.get_node(*old_root_node_id).unwrap();
    assert_eq!(old_root_node.name.as_deref(), Some("Support"));
}

#[test]
fn test_reroot_support_at_internal() {
    // Reroot at the internal node that carries the support value.
    // Original: (A,(B,C)Support)Root;
    // After reroot at Support:
    //   * Support becomes root and receives the old root's name.
    //   * The old root becomes a child of Support and receives the support label.
    let mut tree = Tree::from_newick("(A,(B,C)Support)Root;").unwrap();
    let support_id = tree.get_node_by_name("Support").unwrap();

    tree.reroot_at(support_id, true).unwrap();

    let root = tree.get_root().unwrap();
    assert_eq!(tree.get_node(root).unwrap().name.as_deref(), Some("Root"));

    let support_node_id = tree
        .get_node(root)
        .unwrap()
        .children
        .iter()
        .find(|&&id| tree.get_node(id).unwrap().name.as_deref() == Some("Support"))
        .copied()
        .expect("support label should move to the old root node");
    let support_node = tree.get_node(support_node_id).unwrap();
    assert!(support_node
        .children
        .iter()
        .any(|&id| tree.get_node(id).unwrap().name.as_deref() == Some("A")));
}

#[test]
fn test_reroot_support_multiple_labels() {
    // Reroot at a leaf when the path contains multiple internal support labels.
    // Original: ((A,B)S1,C)S2;
    // Split {A,B}|{C} is supported by S1. After reroot at A, that split is
    // represented by the edge A -> S2, so the S1 label should follow to S2.
    let mut tree = Tree::from_newick("((A,B)S1,C)S2;").unwrap();
    let a_id = tree.get_node_by_name("A").unwrap();

    tree.reroot_at(a_id, true).unwrap();

    assert_eq!(tree.get_root(), Some(a_id));
    let root = tree.get_node(tree.get_root().unwrap()).unwrap();
    assert_eq!(root.name.as_deref(), Some("A"));

    let s1_node_id = tree
        .get_node_by_name("S1")
        .expect("S1 support label should be preserved");
    let s1_node = tree.get_node(s1_node_id).unwrap();
    assert!(!s1_node.is_leaf());
    assert!(s1_node
        .children
        .iter()
        .any(|&id| tree.get_node(id).unwrap().name.as_deref() == Some("C")));
}

#[test]
fn test_reroot_longest_branch() {
    // (A:1, B:2)Root;
    // Longest branch is B (len 2).
    // Reroot should pick B.
    let mut tree = Tree::from_newick("(A:1,B:2)Root;").unwrap();

    let target = tree.get_node_with_longest_edge().unwrap();
    let b_id = tree.get_node_by_name("B").unwrap();
    assert_eq!(target, b_id);

    tree.reroot_at(target, false).unwrap();
    assert_eq!(tree.get_root(), Some(b_id));
}

#[test]
fn test_insert_parent_pair_requires_siblings() {
    let mut tree = Tree::from_newick("((A,B)C,D)Root;").unwrap();
    let a_id = tree.get_node_by_name("A").unwrap();
    let d_id = tree.get_node_by_name("D").unwrap();

    // A and D are not siblings: inserting a common parent must fail.
    assert!(tree.insert_parent_pair(a_id, d_id).is_err());

    let b_id = tree.get_node_by_name("B").unwrap();
    // A and B are siblings under C.
    let new_parent = tree.insert_parent_pair(a_id, b_id).unwrap();
    let new_parent_node = tree.get_node(new_parent).unwrap();
    assert_eq!(new_parent_node.children.len(), 2);
    assert!(new_parent_node.children.contains(&a_id));
    assert!(new_parent_node.children.contains(&b_id));
}

#[test]
fn test_extract_subtree_skips_deleted_children() {
    let mut tree = Tree::from_newick("(A,B)Root;").unwrap();
    let root_id = tree.get_root().unwrap();

    // Simulate a malformed tree where a deleted node id is still listed as a
    // child of the root. Extraction should skip the deleted node rather than
    // panic.
    let deleted_id = tree.add_node();
    tree.get_node_mut(deleted_id).unwrap().deleted = true;
    tree.get_node_mut(root_id)
        .unwrap()
        .children
        .push(deleted_id);

    let subtree = tree.extract_subtree(root_id).unwrap();
    assert_eq!(subtree.len(), 3); // root + A + B, deleted node skipped
}

#[test]
fn nan_length_get_distance() {
    // root -> internal(NaN) -> A(1.0)
    // root -> B(2.0)
    let mut tree = Tree::new();
    let root = tree.add_node();
    let internal = tree.add_node();
    let leaf_a = tree.add_node();
    let leaf_b = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, internal).unwrap();
    tree.add_child(internal, leaf_a).unwrap();
    tree.add_child(root, leaf_b).unwrap();

    tree.get_node_mut(internal).unwrap().length = Some(f64::NAN);
    tree.get_node_mut(leaf_a).unwrap().length = Some(1.0);
    tree.get_node_mut(leaf_b).unwrap().length = Some(2.0);

    let (weighted, topo) = tree.get_distance(leaf_a, leaf_b).unwrap();
    assert!(
        (weighted - 3.0).abs() < 1e-9,
        "expected 3.0, got {}",
        weighted
    );
    assert_eq!(topo, 3);
}

#[test]
fn nan_length_get_height() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let internal = tree.add_node();
    let leaf_a = tree.add_node();
    let leaf_b = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, internal).unwrap();
    tree.add_child(internal, leaf_a).unwrap();
    tree.add_child(root, leaf_b).unwrap();

    tree.get_node_mut(internal).unwrap().length = Some(f64::NAN);
    tree.get_node_mut(leaf_a).unwrap().length = Some(1.0);
    tree.get_node_mut(leaf_b).unwrap().length = Some(2.0);

    let height = tree.get_height(root, true);
    assert!((height - 2.0).abs() < 1e-9, "expected 2.0, got {}", height);
}

#[test]
fn nan_length_compute_node_heights() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let internal = tree.add_node();
    let leaf_a = tree.add_node();
    let leaf_b = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, internal).unwrap();
    tree.add_child(internal, leaf_a).unwrap();
    tree.add_child(root, leaf_b).unwrap();

    tree.get_node_mut(internal).unwrap().length = Some(f64::NAN);
    tree.get_node_mut(leaf_a).unwrap().length = Some(1.0);
    tree.get_node_mut(leaf_b).unwrap().length = Some(2.0);

    let heights = super::stat::compute_node_heights(&tree);
    assert!(
        (heights[&root] - 2.0).abs() < 1e-9,
        "root height expected 2.0, got {}",
        heights[&root]
    );
    assert!(
        (heights[&internal] - 1.0).abs() < 1e-9,
        "internal height expected 1.0, got {}",
        heights[&internal]
    );
    assert_eq!(heights[&leaf_a], 0.0);
    assert_eq!(heights[&leaf_b], 0.0);
}

#[test]
fn nan_length_compute_root_distances() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let internal = tree.add_node();
    let leaf_a = tree.add_node();
    let leaf_b = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, internal).unwrap();
    tree.add_child(internal, leaf_a).unwrap();
    tree.add_child(root, leaf_b).unwrap();

    tree.get_node_mut(internal).unwrap().length = Some(f64::NAN);
    tree.get_node_mut(leaf_a).unwrap().length = Some(1.0);
    tree.get_node_mut(leaf_b).unwrap().length = Some(2.0);

    let dists = super::stat::compute_root_distances(&tree);
    assert_eq!(dists[&root], 0.0);
    assert_eq!(dists[&internal], 0.0);
    assert!(
        (dists[&leaf_a] - 1.0).abs() < 1e-9,
        "leaf_a distance expected 1.0, got {}",
        dists[&leaf_a]
    );
    assert!(
        (dists[&leaf_b] - 2.0).abs() < 1e-9,
        "leaf_b distance expected 2.0, got {}",
        dists[&leaf_b]
    );
}

#[test]
fn nan_length_diameter() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let internal = tree.add_node();
    let leaf_a = tree.add_node();
    let leaf_b = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, internal).unwrap();
    tree.add_child(internal, leaf_a).unwrap();
    tree.add_child(root, leaf_b).unwrap();

    tree.get_node_mut(internal).unwrap().length = Some(f64::NAN);
    tree.get_node_mut(leaf_a).unwrap().length = Some(1.0);
    tree.get_node_mut(leaf_b).unwrap().length = Some(2.0);

    assert!(
        (tree.diameter() - 3.0).abs() < 1e-9,
        "expected 3.0, got {}",
        tree.diameter()
    );
}

#[test]
fn test_has_branch_lengths() {
    let tree = Tree::from_newick("(A,B)Root;").unwrap();
    assert!(!super::stat::has_branch_lengths(&tree));

    let tree = Tree::from_newick("(A:1,B)Root;").unwrap();
    assert!(super::stat::has_branch_lengths(&tree));

    let tree = Tree::from_newick("(A:0,B:0)Root;").unwrap();
    assert!(!super::stat::has_branch_lengths(&tree));

    // Root length is ignored.
    let tree = Tree::from_newick("(A,B)Root:100;").unwrap();
    assert!(!super::stat::has_branch_lengths(&tree));
}

#[test]
fn nan_length_get_node_with_longest_edge() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let leaf_nan = tree.add_node();
    let leaf_long = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, leaf_nan).unwrap();
    tree.add_child(root, leaf_long).unwrap();

    tree.get_node_mut(leaf_nan).unwrap().length = Some(f64::NAN);
    tree.get_node_mut(leaf_long).unwrap().length = Some(5.0);

    assert_eq!(tree.get_node_with_longest_edge(), Some(leaf_long));
}

#[test]
fn nan_length_collapse_node() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let internal = tree.add_node();
    let leaf = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, internal).unwrap();
    tree.add_child(internal, leaf).unwrap();

    tree.get_node_mut(internal).unwrap().length = Some(2.0);
    tree.get_node_mut(leaf).unwrap().length = Some(f64::NAN);

    tree.collapse_node(internal).unwrap();

    assert!(tree.get_node(internal).is_none());
    let leaf_node = tree.get_node(leaf).unwrap();
    assert!(
        (leaf_node.length.unwrap() - 2.0).abs() < 1e-9,
        "expected 2.0, got {:?}",
        leaf_node.length
    );
}

#[test]
fn nan_length_insert_parent() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let leaf = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, leaf).unwrap();
    tree.get_node_mut(leaf).unwrap().length = Some(f64::NAN);

    let new_parent = tree.insert_parent(leaf).unwrap();

    let new_parent_node = tree.get_node(new_parent).unwrap();
    assert!(
        (new_parent_node.length.unwrap()).abs() < 1e-9,
        "expected 0.0, got {:?}",
        new_parent_node.length
    );

    let leaf_node = tree.get_node(leaf).unwrap();
    assert!(
        (leaf_node.length.unwrap()).abs() < 1e-9,
        "expected 0.0, got {:?}",
        leaf_node.length
    );
}

#[test]
fn nan_length_to_dot() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let leaf = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, leaf).unwrap();
    tree.get_node_mut(leaf).unwrap().length = Some(f64::NAN);

    let dot = tree.to_dot();
    let edge_line = dot.lines().find(|l| l.contains("->")).unwrap();
    assert!(
        !edge_line.contains("[label="),
        "NaN edge should not have a label: {}",
        edge_line
    );
}

#[test]
fn deleted_node_get_path_from_root_errors() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let internal = tree.add_node();
    let leaf = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, internal).unwrap();
    tree.add_child(internal, leaf).unwrap();

    // Simulate malformed tree: internal is marked deleted but leaf still points to it.
    tree.get_node_mut(internal).unwrap().deleted = true;

    assert!(tree.get_path_from_root(leaf).is_err());
}

#[test]
fn get_distance_with_deleted_node_on_path() {
    let mut tree = Tree::new();
    //    0
    //   / \
    //  1   2
    // /
    //3
    let n0 = tree.add_node();
    let n1 = tree.add_node();
    let n2 = tree.add_node();
    let n3 = tree.add_node();

    let _ = tree.set_root(n0);
    tree.add_child(n0, n1).unwrap();
    tree.add_child(n0, n2).unwrap();
    tree.add_child(n1, n3).unwrap();

    // Mark n1 as deleted; n3's parent still points to it
    tree.get_node_mut(n1).unwrap().deleted = true;

    // get_distance must return an error (not hang) because the path
    // from n3 to root goes through the deleted n1
    assert!(tree.get_distance(n3, n2).is_err());
}

#[test]
fn test_insert_parent_pair_siblings_preserves_lengths() {
    let mut tree = Tree::from_newick("((A:1.0,B:2.0)C:3.0,D)Root;").unwrap();
    let a_id = tree.get_node_by_name("A").unwrap();
    let b_id = tree.get_node_by_name("B").unwrap();
    let c_id = tree.get_node_by_name("C").unwrap();

    let new_parent = tree.insert_parent_pair(a_id, b_id).unwrap();
    let new_parent_node = tree.get_node(new_parent).unwrap();

    assert!(new_parent_node.children.contains(&a_id));
    assert!(new_parent_node.children.contains(&b_id));
    assert_eq!(new_parent_node.parent, Some(c_id));
    assert_eq!(new_parent_node.length, None);

    assert_eq!(tree.get_node(a_id).unwrap().parent, Some(new_parent));
    assert_eq!(tree.get_node(b_id).unwrap().parent, Some(new_parent));
    assert!((tree.get_node(a_id).unwrap().length.unwrap() - 1.0).abs() < 1e-9);
    assert!((tree.get_node(b_id).unwrap().length.unwrap() - 2.0).abs() < 1e-9);
}

#[test]
fn test_insert_parent_pair_root_children() {
    let mut tree = Tree::from_newick("(A,B)Root;").unwrap();
    let a_id = tree.get_node_by_name("A").unwrap();
    let b_id = tree.get_node_by_name("B").unwrap();
    let root = tree.get_root().unwrap();

    let new_parent = tree.insert_parent_pair(a_id, b_id).unwrap();
    let root_node = tree.get_node(root).unwrap();

    assert_eq!(root_node.children, vec![new_parent]);
    assert_eq!(tree.get_node(new_parent).unwrap().parent, Some(root));
}

#[test]
fn test_compact_preserves_root() {
    let mut tree = Tree::from_newick("((A,B)C,D)Root;").unwrap();
    let c_id = tree.get_node_by_name("C").unwrap();

    tree.remove_node(c_id, true);
    tree.compact();

    let root = tree.get_root().unwrap();
    let root_node = tree.get_node(root).unwrap();
    assert_eq!(root_node.name.as_deref(), Some("Root"));
    assert!(tree.get_node_by_name("A").is_none());
    assert!(tree.get_node_by_name("C").is_none());
    assert!(tree.get_node_by_name("D").is_some());
}

#[test]
fn test_deroot_binary_root_becomes_multifurcating() {
    let mut tree = Tree::from_newick("((A:1,B:2)C:3,D:4)Root;").unwrap();
    tree.deroot().unwrap();

    let root = tree.get_root().unwrap();
    let root_node = tree.get_node(root).unwrap();
    assert_eq!(root_node.children.len(), 3);

    let child_names: Vec<String> = root_node
        .children
        .iter()
        .map(|&id| tree.get_node(id).unwrap().name.clone().unwrap_or_default())
        .collect();
    assert!(child_names.contains(&"A".to_string()));
    assert!(child_names.contains(&"B".to_string()));
    assert!(child_names.contains(&"D".to_string()));
}

#[test]
fn test_remove_degree_two_nodes() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let internal = tree.add_node();
    let leaf = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, internal).unwrap();
    tree.add_child(internal, leaf).unwrap();

    tree.get_node_mut(internal).unwrap().length = Some(2.0);
    tree.get_node_mut(leaf).unwrap().length = Some(3.0);

    tree.remove_degree_two_nodes().unwrap();

    assert!(tree.get_node(internal).is_none());
    let root_node = tree.get_node(root).unwrap();
    assert_eq!(root_node.children, vec![leaf]);

    let leaf_node = tree.get_node(leaf).unwrap();
    assert_eq!(leaf_node.parent, Some(root));
    assert!((leaf_node.length.unwrap() - 5.0).abs() < 1e-9);
}

#[test]
fn test_condense_subtree() {
    let mut tree = Tree::from_newick("((A,B)C,D)Root;").unwrap();
    let c_id = tree.get_node_by_name("C").unwrap();

    tree.condense_subtree(c_id, "Clade", 2).unwrap();

    let root = tree.get_root().unwrap();
    let root_node = tree.get_node(root).unwrap();
    assert_eq!(root_node.children.len(), 2);

    let clade_id = tree.get_node_by_name("Clade").unwrap();
    let clade = tree.get_node(clade_id).unwrap();
    assert!(clade.children.is_empty());
    assert_eq!(clade.get_property("member"), Some(&"2".to_string()));
    assert_eq!(clade.get_property("tri"), Some(&"white".to_string()));

    assert!(tree.get_node_by_name("A").is_none());
    assert!(tree.get_node_by_name("B").is_none());
}

#[test]
fn test_lax_complement_lca() {
    let tree = Tree::from_newick("(A,(B,C)D)Root;").unwrap();
    let a_id = tree.get_node_by_name("A").unwrap();
    let d_id = tree.get_node_by_name("D").unwrap();
    let root = tree.get_root().unwrap();

    let specified: BTreeSet<NodeId> = [a_id].into_iter().collect();
    let comp_lca = super::query::lax_complement_lca(&tree, &specified, root).unwrap();
    assert_eq!(comp_lca, d_id);
}

#[test]
fn add_child_rejects_cycle() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let child = tree.add_node();
    let grandchild = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, child).unwrap();
    tree.add_child(child, grandchild).unwrap();

    // Adding root as child of grandchild would create a cycle.
    assert!(tree.add_child(grandchild, root).is_err());
}

#[test]
fn collapse_node_preserves_none_length_when_both_missing() {
    let mut tree = Tree::new();
    let root = tree.add_node();
    let internal = tree.add_node();
    let leaf = tree.add_node();

    let _ = tree.set_root(root);
    tree.add_child(root, internal).unwrap();
    tree.add_child(internal, leaf).unwrap();

    // Both edges have no length.
    tree.collapse_node(internal).unwrap();

    assert!(tree.get_node(internal).is_none());
    let leaf_node = tree.get_node(leaf).unwrap();
    assert_eq!(
        leaf_node.length, None,
        "expected None when both edges lack length"
    );
}

#[test]
fn node_with_length_rejects_non_positive_and_non_finite() {
    use crate::libs::phylo::Node;

    assert_eq!(Node::new(0).with_length(f64::NAN).length, None);
    assert_eq!(Node::new(0).with_length(f64::INFINITY).length, None);
    assert_eq!(Node::new(0).with_length(f64::NEG_INFINITY).length, None);
    assert_eq!(Node::new(0).with_length(0.0).length, None);
    assert_eq!(Node::new(0).with_length(-1.0).length, None);
    assert_eq!(Node::new(0).with_length(1.0).length, Some(1.0));
}

#[test]
fn test_is_clade() {
    let tree = Tree::from_newick("((A,B)C,D)Root;").unwrap();
    let a = tree.get_node_by_name("A").unwrap();
    let b = tree.get_node_by_name("B").unwrap();
    let d = tree.get_node_by_name("D").unwrap();

    // Empty and single-node sets are not clades.
    assert!(!tree.is_clade(&[]));
    assert!(!tree.is_clade(&[a]));

    // Valid clade.
    assert!(tree.is_clade(&[a, b]));

    // Non-monophyletic pair.
    assert!(!tree.is_clade(&[a, d]));
}

#[test]
fn test_reroot_support_root_child_leaf() {
    // Reroot at a leaf that is a direct child of the old root.
    // Original: (A,(B,C)S1)S2;
    // Path: S2 -> A. A is a leaf, so S2's name (which annotated the
    // non-existent edge above the root) is dropped after reversal. S1 is off
    // the path and keeps its support label.
    let mut tree = Tree::from_newick("(A,(B,C)S1)S2;").unwrap();
    let a_id = tree.get_node_by_name("A").unwrap();

    tree.reroot_at(a_id, true).unwrap();

    assert_eq!(tree.get_root(), Some(a_id));
    let root = tree.get_node(tree.get_root().unwrap()).unwrap();
    assert_eq!(root.name.as_deref(), Some("A"));

    // Old root S2 should have lost its name.
    assert!(tree.get_node_by_name("S2").is_none());

    // S1 should still exist and keep its original support label.
    let s1_id = tree.get_node_by_name("S1").unwrap();
    let s1_node = tree.get_node(s1_id).unwrap();
    assert!(s1_node
        .children
        .iter()
        .any(|&id| tree.get_node(id).unwrap().name.as_deref() == Some("B")));
    assert!(s1_node
        .children
        .iter()
        .any(|&id| tree.get_node(id).unwrap().name.as_deref() == Some("C")));
}

#[test]
fn test_reroot_support_multifurcating_internal() {
    // Reroot at a leaf under a multifurcating internal support node.
    // Original: (A,(B,C,D)S1)S2;
    // Path: S2 -> S1 -> B. S1's label moves to S2; S1 loses its label.
    let mut tree = Tree::from_newick("(A,(B,C,D)S1)S2;").unwrap();
    let b_id = tree.get_node_by_name("B").unwrap();

    tree.reroot_at(b_id, true).unwrap();

    assert_eq!(tree.get_root(), Some(b_id));

    // S2 should now carry the old S1 label.
    let s2_id = tree.get_node_by_name("S1").unwrap();
    let s2_node = tree.get_node(s2_id).unwrap();
    assert!(s2_node
        .children
        .iter()
        .any(|&id| tree.get_node(id).unwrap().name.as_deref() == Some("A")));

    // S1 should now be a child of the new root, with no name, and still multifurcating.
    let root = tree.get_node(tree.get_root().unwrap()).unwrap();
    let s1_id = root
        .children
        .iter()
        .find(|&&id| {
            let n = tree.get_node(id).unwrap();
            n.name.is_none() && n.children.len() == 3
        })
        .copied()
        .unwrap();
    let s1_node = tree.get_node(s1_id).unwrap();
    assert_eq!(s1_node.children.len(), 3);
    assert!(s1_node
        .children
        .iter()
        .any(|&id| tree.get_node(id).unwrap().name.as_deref() == Some("C")));
    assert!(s1_node
        .children
        .iter()
        .any(|&id| tree.get_node(id).unwrap().name.as_deref() == Some("D")));
}

#[test]
fn test_reroot_at_root_is_noop() {
    let mut tree = Tree::from_newick("(A,(B,C)S1)S2;").unwrap();
    let root_id = tree.get_root().unwrap();
    let original = tree.to_newick();

    tree.reroot_at(root_id, true).unwrap();

    assert_eq!(tree.get_root(), Some(root_id));
    assert_eq!(tree.to_newick(), original);
}
