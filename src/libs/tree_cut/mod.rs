use crate::libs::pairmat::NamedMatrix;
use crate::libs::phylo::node::NodeId;
use crate::libs::phylo::tree::Tree;
use anyhow::Result;
use std::collections::HashMap;

pub mod clade;
pub mod dynamic;
pub mod hybrid;
pub mod inconsistent;
pub mod link;
pub mod method;
pub mod partition;
pub mod scan;
pub mod simple;

pub use method::{build_method, Method, METHOD_NAMES};
pub use partition::{
    find_representative, format_clusters, format_scan_rows, partition_to_clusters,
    Cluster, Partition, RepMode,
};

use dynamic::{cutree_dynamic_tree, DynamicTreeOptions};
use hybrid::{cutree_hybrid, HybridOptions};

/// Cut-method dispatch value (clap-free). The caller constructs this from
/// `ArgMatches` and `dispatch_cut` executes the corresponding algorithm.
pub enum CutDispatch {
    /// Dynamic Tree Cut.
    DynamicTree(DynamicTreeOptions),
    /// Dynamic Hybrid Cut (with pre-loaded distance matrix).
    DynamicHybrid(HybridOptions),
    /// One of the standard `METHOD_NAMES` methods.
    Standard {
        /// Method name (must be one of `METHOD_NAMES`).
        name: &'static str,
        /// Threshold value for the method.
        val: f64,
        /// Depth used by the inconsistent-coefficient method.
        deep: usize,
        /// Precomputed leaf depth statistics for `leaf_dist_*` methods.
        leaf_depths: Option<(f64, f64, f64)>,
    },
}

/// Build a `CutDispatch` from raw CLI-style values.
///
/// Priority: `dynamic_tree` > `dynamic_hybrid` > standard `method_name`.
/// For standard methods, `val` is the threshold; `leaf_dist_*` methods will
/// compute leaf depth stats from `tree` automatically.
#[allow(clippy::too_many_arguments)]
pub fn build_dispatch(
    tree: &Tree,
    method_name: Option<&'static str>,
    val: f64,
    deep: usize,
    dynamic_tree: Option<usize>,
    dynamic_hybrid: Option<usize>,
    max_tree_height: Option<f64>,
    deep_split: bool,
    no_pam_dendro: bool,
    max_pam_dist: Option<f64>,
    matrix: Option<NamedMatrix>,
) -> Result<CutDispatch> {
    if let Some(min_module_size) = dynamic_tree {
        if min_module_size == 0 {
            anyhow::bail!("dynamic-tree min cluster size must be greater than 0");
        }
        return Ok(CutDispatch::DynamicTree(DynamicTreeOptions {
            min_module_size,
            deep_split,
            max_tree_height,
        }));
    }

    if let Some(min_cluster_size) = dynamic_hybrid {
        if min_cluster_size == 0 {
            anyhow::bail!("dynamic-hybrid min cluster size must be greater than 0");
        }
        let dist_matrix = matrix
            .ok_or_else(|| anyhow::anyhow!("--matrix is required for dynamic-hybrid"))?;
        return Ok(CutDispatch::DynamicHybrid(HybridOptions {
            min_cluster_size,
            dist_matrix,
            cut_height: max_tree_height,
            deep_split: if deep_split { 1 } else { 0 },
            max_core_scatter: None,
            min_gap: None,
            pam_stage: true,
            pam_respects_dendro: !no_pam_dendro,
            max_pam_dist,
            respect_small_clusters: true,
        }));
    }

    let name = method_name.ok_or_else(|| anyhow::anyhow!("no cut method specified"))?;
    let leaf_depths = if name.starts_with("leaf_dist_") {
        Some(crate::libs::phylo::tree::stat::get_leaf_depth_stats(tree))
    } else {
        None
    };
    Ok(CutDispatch::Standard {
        name,
        val,
        deep,
        leaf_depths,
    })
}

/// Execute the cut specified by `dispatch` on `tree`. Returns the resulting
/// partition and the method name (for labeling).
pub fn dispatch_cut(
    tree: &Tree,
    dispatch: CutDispatch,
) -> Result<(Partition, &'static str)> {
    match dispatch {
        CutDispatch::DynamicTree(opts) => {
            let p = cutree_dynamic_tree(tree, opts)?;
            Ok((p, "dynamic-tree"))
        }
        CutDispatch::DynamicHybrid(opts) => {
            let p = cutree_hybrid(tree, opts)?;
            Ok((p, "dynamic-hybrid"))
        }
        CutDispatch::Standard {
            name,
            val,
            deep,
            leaf_depths,
        } => {
            let method = build_method(name, val, deep, leaf_depths)?;
            let p = cut(tree, method)?;
            Ok((p, name))
        }
    }
}

/// Cut the tree according to the specified method.
///
/// # Arguments
///
/// *   `tree` - The phylogenetic tree to cut.
/// *   `method` - The cutting method/strategy.
///
/// # Returns
///
/// A `Result` containing the `Partition` or an error message.
pub fn cut(tree: &Tree, method: Method) -> Result<Partition> {
    if tree.is_empty() {
        return Ok(Partition::new());
    }

    match method {
        Method::K(k) => simple::cut_k(tree, k),
        Method::Height(h) => simple::cut_height(tree, h),
        Method::RootDist(d) => simple::cut_root_dist(tree, d),
        Method::MaxClade(t) => clade::cut_max_clade(tree, t),
        Method::AvgClade(t) => clade::cut_avg_clade(tree, t),
        Method::MedClade(t) => clade::cut_med_clade(tree, t),
        Method::SumBranch(t) => clade::cut_sum_branch(tree, t),
        Method::Inconsistent(t, d) => inconsistent::cut_inconsistent(tree, t, d),
        Method::SingleLinkage(t) => link::cut_single_linkage(tree, t),
    }
}

// --- Helpers ---

/// Compute max distance from each node to its leaves
pub(crate) fn compute_heights(
    tree: &Tree,
    root: NodeId,
) -> Result<HashMap<NodeId, f64>> {
    let mut heights = HashMap::new();
    let post_order = crate::libs::phylo::tree::traversal::postorder(tree, root);

    for id in post_order {
        let node = tree
            .get_node(id)
            .ok_or_else(|| anyhow::anyhow!("node {} not found", id))?;
        if node.children.is_empty() {
            heights.insert(id, 0.0);
        } else {
            let mut max_h = 0.0;
            for &child in &node.children {
                let child_node = tree
                    .get_node(child)
                    .ok_or_else(|| anyhow::anyhow!("node {} not found", child))?;
                let len = child_node.finite_length();
                let h = heights.get(&child).copied().unwrap_or(0.0) + len;
                if h > max_h {
                    max_h = h;
                }
            }
            heights.insert(id, max_h);
        }
    }
    Ok(heights)
}

/// Assign leaves to clusters based on cluster roots
pub(crate) fn assign_clusters(
    tree: &Tree,
    cluster_roots: Vec<NodeId>,
) -> Result<Partition> {
    let mut part = Partition::new();
    let mut cluster_id = 0;

    for root in cluster_roots {
        cluster_id += 1;
        let leaves = crate::libs::phylo::tree::stat::get_leaves(tree, root);
        for leaf in leaves {
            part.assignment.insert(leaf, cluster_id);
        }
    }

    part.num_clusters = cluster_id;
    Ok(part)
}

/// Branch length used to mask low-support edges.
///
/// Must be finite so that `Node::finite_length()` (used by stat/balance/query
/// modules) propagates it as-is rather than collapsing it to 0.0. The value
/// 1e20 is large enough to exceed any realistic cut threshold while staying
/// within the finite range (1e20 + 1e20 = 2e20, no overflow).
const SUPPORT_FILTER_SENTINEL: f64 = 1e20;

/// Mask low-support internal nodes by setting branch length to a sentinel
/// value that behaves as "effectively infinite" across all cut methods
/// (TreeCluster semantics).
///
/// Uses a finite sentinel (`SUPPORT_FILTER_SENTINEL`) rather than `f64::INFINITY`
/// because `Node::finite_length()` — used by `stat::compute_node_heights`,
/// `balance::compute_avg_clade_distances`, `query::get_height`, etc. — maps
/// non-finite lengths to `0.0`, which would silently disable the filter for
/// `--avg-clade`, `--inconsistent`, `--dynamic-tree`, and `--dynamic-hybrid`.
pub fn apply_support_filter(tree: &mut Tree, threshold: f64) {
    let root = match tree.get_root() {
        Some(r) => r,
        None => return,
    };

    // First pass: collect internal node IDs whose support is below the threshold.
    // A tree traversal is used instead of `0..tree.len()` so the function does not
    // rely on node IDs being contiguous.
    let mut to_mask = Vec::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        if let Some(node) = tree.get_node(id) {
            if !node.children.is_empty() {
                let support = node
                    .name
                    .as_ref()
                    .and_then(|n| n.parse::<f64>().ok())
                    .unwrap_or(100.0);
                if support < threshold {
                    to_mask.push(id);
                }
            }
            for &child in &node.children {
                stack.push(child);
            }
        }
    }

    // Second pass: apply the mask. This is kept separate so the first pass can
    // hold a shared borrow of the tree.
    for id in to_mask {
        if let Some(node) = tree.get_node_mut(id) {
            node.length = Some(SUPPORT_FILTER_SENTINEL);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_tree() -> Tree {
        Tree::from_newick("((A:0.1,B:0.1):0.1,C:0.2);").expect("valid newick")
    }

    #[test]
    fn test_build_dispatch_dynamic_tree_zero_rejected() {
        let tree = tiny_tree();
        let result = build_dispatch(
            &tree,
            None,
            0.0,
            2,
            Some(0),
            None,
            None,
            false,
            false,
            None,
            None,
        );
        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(
            msg.contains("dynamic-tree min cluster size must be greater than 0"),
            "got: {}",
            msg
        );
    }

    #[test]
    fn test_build_dispatch_dynamic_hybrid_zero_rejected() {
        let tree = tiny_tree();
        // Condensed upper-triangle values for A-B, A-C, B-C.
        let matrix = NamedMatrix::new_from_values(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![0.2, 1.0, 1.0],
        )
        .expect("valid matrix");
        let result = build_dispatch(
            &tree,
            None,
            0.0,
            2,
            None,
            Some(0),
            None,
            false,
            false,
            None,
            Some(matrix),
        );
        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(
            msg.contains("dynamic-hybrid min cluster size must be greater than 0"),
            "got: {}",
            msg
        );
    }
}
