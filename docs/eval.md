# necom eval

`necom eval` provides evaluation metrics for clustering partitions and phylogenetic trees. It is the unified entry point for assessing result quality, support, and consistency.

## Subcommands

Subcommands are grouped by evaluation target:

*   **Tree comparison**:
    *   `compare`: Compare trees (RF, WRF, KF distances).

---

## Tree Comparison

### compare

Compare trees using Robinson-Foulds (RF) distance and its variants. Supports pairwise comparison within a single file and cross-comparison between two files.

---

## Branch Length Handling

`necom eval compare` treats non-finite branch lengths (`NaN`, positive/negative infinity), negative values, and zero values as `0.0` during computation. This normalization prevents invalid values from polluting WRF and KF distance computations. Input files themselves are not modified.

---

## Planned Subcommands

*   `partition`: Clustering partition evaluation (migrated from `cl