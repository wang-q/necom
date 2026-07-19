use super::Partition;
use crate::libs::phylo::node::NodeId;
use crate::libs::phylo::tree::stat::compute_node_heights;
use crate::libs::phylo::tree::Tree;
use std::collections::HashMap;

/// Maximum iterations for the dynamic tree deep-split loop. Acts as a safety
/// break against pathological non-convergence; normal inputs converge in <10.
const MAX_DEEP_SPLIT_ITERATIONS: usize = 100;

/// Parameters for Dynamic Tree Cut.
pub struct DynamicTreeOptions {
    /// Minimum number of leaves in a cluster module.
    pub min_module_size: usize,
    /// Whether to enable iterative deep splitting.
    pub deep_split: bool,
    /// Optional upper bound on joining height (defaults to 99% of tree height).
    pub max_tree_height: Option<f64>,
}

impl Default for DynamicTreeOptions {
    fn default() -> Self {
        Self {
            min_module_size: 50,
            deep_split: true,
            max_tree_height: None,
        }
    }
}

/// Main entry point for dynamic tree cut
pub fn cutree_dynamic_tree(
    tree: &Tree,
    options: DynamicTreeOptions,
) -> anyhow::Result<Partition> {
    if tree.is_empty() {
        return Ok(Partition::new());
    }

    let root = match tree.get_root() {
        Some(r) => r,
        None => return Ok(Partition::new()),
    };

    // 1. Build Height Sequence
    // Leaves must follow the dendrogram left-to-right order, which matches a
    // preorder traversal. `tree.get_leaves()` returns a stack-based order and
    // would break the contiguous-segment assumption of dynamic tree cut.
    let leaves: Vec<NodeId> = crate::libs::phylo::tree::traversal::preorder(tree, root)
        .into_iter()
        .filter(|&id| {
            tree.get_node(id)
                .map(|n| n.children.is_empty())
                .unwrap_or(false)
        })
        .collect();

    if leaves.is_empty() {
        return Ok(Partition::new());
    }

    // Pre-calculate heights for all nodes using weighted finite branch lengths.
    let node_heights = compute_node_heights(tree);

    let max_h = *node_heights.get(&root).unwrap_or(&0.0);
    let cut_height = options.max_tree_height.unwrap_or(0.99 * max_h);

    // 2. Initial Static Cut
    let mut height_seq = Vec::with_capacity(leaves.len());
    for &leaf_id in &leaves {
        let parent = tree.get_node(leaf_id).and_then(|n| n.parent);
        let h = match parent {
            Some(p_id) => *node_heights.get(&p_id).unwrap_or(&0.0),
            None => max_h, // Root leaf
        };
        height_seq.push((leaf_id, h));
    }

    // 3. Identify initial clusters (contiguous segments below cut_height)
    let static_clusters =
        cutere_static(tree, &node_heights, cut_height, options.min_module_size)?;

    // 4. Process each cluster
    let mut final_partition = HashMap::new();

    // Group leaves by static cluster
    let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new(); // ClusterID -> Vec<Index in height_seq>
    for (i, (leaf_id, _)) in height_seq.iter().enumerate() {
        if let Some(&cid) = static_clusters.get(leaf_id) {
            if cid > 0 {
                clusters.entry(cid).or_default().push(i);
            } else {
                final_partition.insert(*leaf_id, 0);
            }
        } else {
            final_partition.insert(*leaf_id, 0);
        }
    }

    let mut next_cluster_id = 1;

    // Sort cluster IDs to ensure deterministic order
    let mut sorted_cids: Vec<_> = clusters.keys().cloned().collect();
    sorted_cids.sort();

    for cid in sorted_cids {
        let indices = &clusters[&cid];
        let cluster_heights: Vec<f64> =
            indices.iter().map(|&i| height_seq[i].1).collect();

        // Use a queue to handle deep split (iterative approach instead of recursion on data)
        // Initial segment is full range [0, len)
        let mut segments = vec![(0, cluster_heights.len())];

        // Iterative processing (loop until stable or deepSplit is false)
        // R logic: outer loop iterates over clusters. Inner loop processes each.
        // If split occurs, new clusters replace old ones in the list.
        // We simulate this with a queue.

        // But R's `cutreeDynamicTree` loop is:
        // while(1==1) {
        //   clupos = NULL
        //   for i in clusters:
        //      iclupos = .processIndividualCluster(...)
        //      clupos.append(iclupos)
        //   if deepSplit == FALSE: break
        //   if no change: break
        //   clusters = clupos
        // }

        let mut loop_count = 0;
        loop {
            loop_count += 1;
            let mut new_segments = Vec::new();
            let mut changed = false;

            for (start, end) in segments {
                let sub_heights = &cluster_heights[start..end];
                let sub_ranges =
                    process_individual_cluster(sub_heights, options.min_module_size);

                // If sub_ranges has more than 1 element, we split.
                // If sub_ranges has 1 element and it's smaller than input, we trimmed?
                // R's `processIndividualCluster` returns ranges.

                if sub_ranges.len() > 1 {
                    changed = true;
                }
                // Also check if range changed (trimmed)
                if sub_ranges.len() == 1 {
                    let (s, e) = sub_ranges[0];
                    if s != 0 || e != sub_heights.len() {
                        changed = true;
                    }
                }

                for (s, e) in sub_ranges {
                    new_segments.push((start + s, start + e));
                }
            }

            segments = new_segments;

            if !options.deep_split || !changed {
                break;
            }
            if loop_count > MAX_DEEP_SPLIT_ITERATIONS {
                break;
            } // Safety break
        }

        // Assign global IDs
        for (start, end) in segments {
            let len = end - start;
            if len >= options.min_module_size {
                let global_id = next_cluster_id;
                next_cluster_id += 1;

                for original_idx in indices.iter().take(end).skip(start) {
                    let leaf_id = height_seq[*original_idx].0;
                    final_partition.insert(leaf_id, global_id);
                }
            } else {
                // Too small after split -> 0
                for original_idx in indices.iter().take(end).skip(start) {
                    let leaf_id = height_seq[*original_idx].0;
                    final_partition.insert(leaf_id, 0);
                }
            }
        }
    }

    // Wrap in Partition struct
    // The previous dynamic_tree used Partition as type alias for HashMap
    // The new Partition is a struct

    // Cluster IDs are assigned sequentially from 1, so the count equals the
    // last assigned ID. When no clusters are formed, next_cluster_id stays 1.
    let num_clusters = next_cluster_id - 1;

    Ok(Partition {
        assignment: final_partition,
        num_clusters,
    })
}

// --- Helpers ---

/// Simulates static cut (cutree).
/// Returns Map: LeafID -> ClusterID. 0 for too small clusters.
fn cutere_static(
    tree: &Tree,
    node_heights: &HashMap<NodeId, f64>,
    cut_height: f64,
    min_size: usize,
) -> anyhow::Result<HashMap<NodeId, usize>> {
    let root = tree
        .get_root()
        .ok_or_else(|| anyhow::anyhow!("Tree has no root"))?;
    let mut cluster_roots = Vec::new();
    let mut stack = vec![root];

    while let Some(node_id) = stack.pop() {
        let h = *node_heights.get(&node_id).unwrap_or(&0.0);
        if h <= cut_height {
            cluster_roots.push(node_id);
        } else {
            if let Some(node) = tree.get_node(node_id) {
                if node.children.is_empty() {
                    cluster_roots.push(node_id);
                } else {
                    for &child in &node.children {
                        stack.push(child);
                    }
                }
            }
        }
    }

    let mut partition = HashMap::new();
    let mut next_id = 1;

    for root_id in cluster_roots {
        let subtree_leaves = crate::libs::phylo::tree::stat::get_leaves(tree, root_id);
        if subtree_leaves.len() >= min_size {
            let cid = next_id;
            next_id += 1;
            for leaf in subtree_leaves {
                partition.insert(leaf, cid);
            }
        } else {
            for leaf in subtree_leaves {
                partition.insert(leaf, 0);
            }
        }
    }

    Ok(partition)
}

/// Returns a list of [start, end) indices relative to input heights.
fn process_individual_cluster(
    heights: &[f64],
    min_module_size: usize,
) -> Vec<(usize, usize)> {
    if heights.len() < min_module_size {
        return vec![(0, heights.len())]; // Return as is, let caller handle size check
    }
    recursive_process(heights, min_module_size, 0)
}

fn recursive_process(
    heights: &[f64],
    min_module_size: usize,
    use_mean_mode: i32, // 0=Normal, 1=High, -1=Low
) -> Vec<(usize, usize)> {
    let n = heights.len();
    if n == 0 {
        return vec![];
    }

    let max_h = heights.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_h = heights.iter().cloned().fold(f64::INFINITY, f64::min);
    let sum_h: f64 = heights.iter().sum();
    let mean_h = sum_h / n as f64;

    let comp_h = match use_mean_mode {
        1 => (mean_h + max_h) / 2.0,
        -1 => (mean_h + min_h) / 2.0,
        _ => mean_h,
    };

    let mut split_indices = Vec::new();

    // Scan for low->high transitions
    // heights[i] < comp_h AND heights[i+1] > comp_h
    for i in 0..n - 1 {
        let h_curr = heights[i];
        let h_next = heights[i + 1];
        if h_curr < comp_h && h_next > comp_h {
            split_indices.push(i);
        }
    }

    if split_indices.is_empty() {
        if use_mean_mode == 0 {
            return recursive_process(heights, min_module_size, 1);
        } else if use_mean_mode == 1 {
            return recursive_process(heights, min_module_size, -1);
        } else {
            return vec![(0, n)];
        }
    }

    let min_tail_len = (min_module_size / 3) + 1;
    let mut valid_splits = Vec::new();

    for &idx in &split_indices {
        // Look backwards from idx
        let mut run_len = 0;
        for k in (0..=idx).rev() {
            if heights[k] < comp_h {
                run_len += 1;
            } else {
                break;
            }
        }

        if run_len >= min_tail_len {
            valid_splits.push(idx);
        }
    }

    let mut boundaries = vec![0];
    for &split in &valid_splits {
        boundaries.push(split + 1);
    }
    boundaries.push(n);

    let min_attach_size = 2 * min_module_size;
    let mut segments: Vec<(usize, usize)> =
        boundaries.windows(2).map(|w| (w[0], w[1])).collect();

    // Iterative merge pass
    let mut changed = true;
    while changed {
        changed = false;
        if segments.len() <= 1 {
            break;
        }

        let mut new_segments = Vec::new();
        let mut i = 0;
        while i < segments.len() {
            if i == segments.len() - 1 {
                new_segments.push(segments[i]);
                break;
            }

            let (s1, e1) = segments[i]; // Left
            let (s2, e2) = segments[i + 1]; // Right (Current)

            let size1 = e1 - s1;
            let size2 = e2 - s2;

            let mean1 = heights[s1..e1].iter().sum::<f64>() / size1 as f64;
            let mean2 = heights[s2..e2].iter().sum::<f64>() / size2 as f64;

            // Merge condition: Right is smaller than Left (height wise) and small in size?
            // R: if( (cur.module.hei<pre.module.hei)&(cur.module.size<cminAttachModuleSize) )

            if mean2 < mean1 && size2 < min_attach_size {
                // Merge 2 into 1
                new_segments.push((s1, e2));
                changed = true;
                i += 2; // Skip next
            } else {
                new_segments.push(segments[i]);
                i += 1;
            }
        }
        segments = new_segments;
    }

    // Tail check
    if segments.len() > 1 {
        let last = segments[segments.len() - 1];
        let prev = segments[segments.len() - 2];
        if (last.1 - last.0) < min_module_size {
            // Merge last into prev
            let new_last = (prev.0, last.1);
            segments.pop();
            segments.pop();
            segments.push(new_last);
        }
    }

    if segments.len() == 1 {
        if use_mean_mode == 0 {
            return recursive_process(heights, min_module_size, 1);
        } else if use_mean_mode == 1 {
            return recursive_process(heights, min_module_size, -1);
        } else {
            return segments;
        }
    }

    segments
}
