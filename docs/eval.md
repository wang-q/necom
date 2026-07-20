# necom eval

`necom eval` provides evaluation metrics for clustering partitions and phylogenetic trees. It is the unified entry point for assessing result quality, support, and consistency.

## Subcommands

Subcommands are grouped by evaluation target:

*   **Tree comparison**:
    *   `compare`: Compare trees (RF, WRF, KF distances).
*   **Partition evaluation**:
    *   `partition`: Evaluate clustering partitions (external and internal metrics).
*   **Branch support**:
    *   `replicate`: Assign support values from replicate trees.

---

## Tree Comparison

### compare

Compare trees using Robinson-Foulds (RF) distance and its variants. Supports pairwise comparison within a single file and cross-comparison between two files.

---

## Partition Evaluation

### partition

Evaluate clustering partition quality. Supports external comparison to a reference partition (ARI, AMI, V-Measure, FMI, NMI, Jaccard, etc.) and internal evaluation using a distance matrix, tree, or coordinate matrix (Silhouette, Dunn, Davies-Bouldin, Calinski-Harabasz, etc.). Batch mode evaluates multiple partitions from parameter scans.

Input validation is strict: empty partitions are rejected, external evaluation requires matching sample sets, and internal evaluation requires every partition sample to be present in the supplied matrix, tree, or coordinate file. These checks prevent silent sample dropping and misleading `NaN` metrics.

See [`docs/eval-partition.md`](eval-partition.md) for detailed metric definitions and the selection guide.

---

## Branch Support

### replicate

Assign support values to internal nodes of a target tree based on replicate trees (e.g., from bootstrap or jackknife resampling). Outputs the annotated tree (Newick). All trees must share the same leaf set.

---

## Branch Length Handling

`necom eval compare` treats non-finite branch lengths (`NaN`, positive/negative infinity), negative values, and zero values as `0.0` during computation. This normalization prevents invalid values from polluting WRF and KF distance computations. Input files themselves are not modified.

---

## Planned Subcommands

The following subcommands are **not yet implemented**. Their CLI surface is not yet finalized; the descriptions below reflect the current design direction, not commitments. See [`notes/design/eval-planned.md`](../notes/design/eval-planned.md) for design status.

### tree (Not Implemented)

Multi-dimensional evaluation of a single phylogenetic tree. The tree is the positional primary input; a partition (`--part`), reference tree (`--ref`), trait map (`--traits`), or original distance matrix (`--dist`) may be supplied as auxiliary context.

Four evaluation dimensions:

*   **Geometry**: Silhouette, Diameter, AvgDist, MinInterDist. Branches on whether the partition corresponds to tree clades — O(N) for clade groups (e.g., `necom cut` output), O(N²) for arbitrary partitions (e.g., dbscan/mcl output).
*   **Trait**: Purity, Entropy, DominantTrait — consistency with external labels (taxonomy, geography, phenotype).
*   **Phylo**: Local RF to reference tree, Monophyly Check, ConflictScore.
*   **Fit**: Cophenetic Correlation against an original distance matrix.

### boot (Not Implemented)

Multiscale bootstrap (pvclust-style) support values for hierarchical clustering. Takes an observation matrix, resamples features at multiple scales, rebuilds the hierarchical tree each time, and fits BP/AU/SI values for every internal node. Design details are in [`notes/design/eval-boot.md`](../notes/design/eval-boot.md).

### quartet (Future Candidate, Not Designed)

Quartet-based consistency or support values for branches. **Not yet designed; the namespace is not reserved.** Boundary: if the goal is to *infer* a new tree from quartets, it belongs to `nwk`; if the goal is to *assess* branch support of a given tree, it belongs to `eval`.

---

## Overlapping Features and Boundaries

Several planned and existing subcommands address superficially similar questions. This section clarifies the boundaries to help choose the right command.

### `eval tree` vs `eval partition --tree`

Both can compute Silhouette from tree distances — in fact `eval tree` reuses the same `silhouette_score` implementation as `eval partition` via the `TreeDistance` adapter. The difference is the **primary subject** and the **scope of metrics**:

| Aspect | `eval partition --tree` | `eval tree --part` |
| :--- | :--- | :--- |
| Primary input | Partition (groups are the subject) | Tree (tree is the subject) |
| Tree role | Distance source (auxiliary) | Evaluation target (primary) |
| Grouping role | Evaluation subject (primary) | Auxiliary context |
| Silhouette | Yes | Yes (same implementation) |
| Clade-aware optimization | No | Yes (O(N) for clade groups) |
| External metrics (ARI, AMI, V-Measure, ...) | Yes (`--other`) | No |
| Coordinate metrics (DBI, CH, ...) | Yes (`--coords`) | No |
| Cophenetic fit to original matrix | No | Yes (`--dist`) |
| Trait purity / entropy | No | Yes (`--traits`) |
| Reference tree comparison | No | Yes (`--ref`) |
| Batch mode (`--input-format long`) | Yes | No |

**Rule of thumb**: If the question is "how good is this clustering?", use `eval partition`. If the question is "how good is this tree, possibly in the context of some grouping?", use `eval tree`.

### `eval tree --ref` vs `eval compare`

Both compute Robinson-Foulds distances, but at different scopes and for different purposes:

*   `eval compare`: **Global** tree-to-tree topology comparison. Takes two trees (or two sets of trees), computes RF/WRF/KF on the full leaf sets. No grouping concept. Supports batch cross-comparison between files.
*   `eval tree --ref`: **Local** comparison in the context of a partition. The reference tree (typically a species tree) provides biological ground truth. Computes Local RF (cluster subtree vs reference subset) and Monophyly Check (do cluster members form a clade on the reference?). Requires leaf-set intersection pruning as a prerequisite, because gene trees usually sample only a subset of the species tree's taxa.

**Rule of thumb**: If comparing two trees as a whole, use `eval compare`. If assessing whether a gene tree's clusters are consistent with a species tree, use `eval tree --ref`.

### `eval quartet` (future) vs `eval compare`

*   `eval compare`: Topology-based (split-set symmetric difference). Pure structural comparison of whole trees.
*   `eval quartet` (future): Quartet-based, can incorporate sequence alignment. Assesses branch-level support rather than overall distance.

**Rule of thumb**: `eval compare` answers "how different are these two trees?"; `eval quartet` answers "how well-supported is each branch of this tree?".

### `eval replicate` vs `eval compare`

*   `eval compare`: Pairwise comparison of given trees. Outputs TSV with RF/WRF/KF distances — a **global** tree-to-tree difference measure.
*   `eval replicate`: Assigns **branch-level** support values to a target tree from replicate trees. Outputs the annotated tree (Newick), not a distance table.

**Rule of thumb**: `eval compare` measures whole-tree topological difference; `eval replicate` measures per-branch support from replicates.

### Parameter naming: `--part` vs `--tree`

The same option names appear under different subcommands with **reversed roles**, reflecting the primary-subject difference. This is intentional but can be confusing:

*   `eval tree --part <FILE>`: the tree is the positional primary; `--part` supplies an optional grouping to evaluate on the tree.
*   `eval partition --tree <FILE>`: the partition is the positional primary; `--tree` supplies an optional distance source for internal metrics.

The option always refers to the *auxiliary* input. When in doubt, identify which input is the positional argument — that is the subject of the evaluation.

---

## History

*   `eval partition` was migrated from `necom clust eval` (2026-07).
*   `eval compare` was migrated from `necom nwk compare` (2026-07).
*   `eval replicate` was migrated from `necom nwk support` (2026-07).