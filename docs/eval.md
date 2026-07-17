# necom eval

`necom eval` provides evaluation metrics for clustering partitions and phylogenetic trees. It is the unified entry point for assessing result quality, support, and consistency.

## Subcommands

Subcommands are grouped by evaluation target:

*   **Tree comparison**:
    *   `compare`: Compare trees (RF, WRF, KF distances).
*   **Partition evaluation**:
    *   `partition`: Evaluate clustering partitions (external and internal metrics).

---

## Tree Comparison

### compare

Compare trees using Robinson-Foulds (RF) distance and its variants. Supports pairwise comparison within a single file and cross-comparison between two files.

---

## Partition Evaluation

### partition

Evaluate clustering partition quality. Supports external comparison to a reference partition (ARI, AMI, V-Measure, FMI, NMI, Jaccard, etc.) and internal evaluation using a distance matrix, tree, or coordinate matrix (Silhouette, Dunn, Davies-Bouldin, Calinski-Harabasz, etc.). Batch mode evaluates multiple partitions from parameter scans.

See [`docs/eval-partition.md`](eval-partition.md) for detailed metric definitions and the selection guide.

---

## Branch Length Handling

`necom eval compare` treats non-finite branch lengths (`NaN`, positive/negative infinity), negative values, and zero values as `0.0` during computation. This normalization prevents invalid values from polluting WRF and KF distance computations. Input files themselves are not modified.

---

## Planned Subcommands

*   `tree`: Multi-dimensional tree evaluation (geometry, taxonomy, evolution). See [`notes/design/eval.md`](../notes/design/eval.md) §4 for the design.