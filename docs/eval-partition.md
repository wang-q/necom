# necom eval partition

This document describes the design philosophy, metrics, and selection guidance for `necom eval partition`. For command-line options and usage examples, see [`docs/help/eval/partition.md`](help/eval/partition.md).

## Design Philosophy

`necom` adopts a **componentized** design philosophy, separating cluster **generation** from **evaluation**. This differs from the integrated design of the Python package `clusteval`:

*   **Python `clusteval`**: The `fit()` method internally performs Grid Search (trying different $k$ or $\epsilon$), computes internal metrics (e.g., Silhouette), and returns the optimal result.
*   **`necom` workflow**:
    1.  **Generate**: Use `necom cut --scan` to generate a series of candidate partitions (`necom clust dbscan`'s `--scan` is not yet implemented).
    2.  **Evaluate**: Use `necom eval partition` to compute evaluation metrics for these candidates in batch.
    3.  **Decide**: The user selects the optimal parameters based on metrics (e.g., Silhouette peak, Elbow point).

* **Complementary**:
  * `necom eval tree` [planned]: Focuses on consistency between tree structure and grouping (geometry/evolution).
  * `necom eval partition`: Focuses on the statistical validity of partitions, supporting external (two-group comparison) and internal (single group + matrix/coordinates/tree) evaluation.
* **Scenarios**:
  * **Algorithm comparison**: Compare result differences between MCL and K-Medoids on the same dataset.
  * **Benchmarking**: Compare clustering results against known standard classifications (Ground Truth) to compute accuracy.
  * **Parameter tuning**: Compare stability of clustering results under different parameters (e.g., `eps` or `inflation`).

This design allows evaluation tools to exist independently of clustering algorithms, supporting clustering results from any source.

## Core Metrics

Clustering evaluation metrics usually fall into two categories: **external validity** (requires Ground Truth or a reference partition) and **internal validity** (depends only on the geometry/statistics of the data).

### 1. External Validity
*Used to compare consistency between two clustering results, or to evaluate how well a result matches the true classification.*

#### 1.1 Pair-Based
*Focuses on whether sample pairs remain in the same/different groups across two partitions.*

- **ARI (Adjusted Rand Index)**
  - **Definition**: Rand Index corrected for chance.
  - **Principle**: Counts sample pairs that are in the same or different groups in both partitions, and subtracts the expected value under random assignment.
  - **Range**: `[-1, 1]`. 1 means perfect agreement, 0 means random level, negative values mean worse than random.
  - **Advantages**:
    - **Interpretable**: 0 is the random baseline, intuitive.
    - **Symmetric**: `ARI(A, B) == ARI(B, A)`.
  - **Disadvantages**: Insensitive to internal cluster structure (e.g., shape).
  - **Applicable**: General scenarios with imbalanced cluster sizes and many clusters.

- **RI (Rand Index)**
  - **Definition**: Proportion of correctly classified sample pairs.
  - **Range**: `[0, 1]`.
  - **Disadvantages**: Not corrected for chance. As the number of clusters increases, the RI of random partitions also approaches 1, reducing discriminative power. Generally **not recommended** for standalone use.

- **Jaccard Index (Pair-wise)**
  - **Definition**: Among all sample pairs considered in the same group by either partition (TP + FP + FN), the proportion considered in the same group by both partitions (TP).
  - **Formula**: $J = \frac{TP}{TP + FP + FN}$.
  - **Meaning**: The Jaccard similarity between set $S_1$ (same-group pairs in P1) and set $S_2$ (same-group pairs in P2).

- **Precision (Pair-wise)**
  - **Definition**: Among all sample pairs considered in the same group by the predicted partition (P1), the proportion also considered in the same group by the true partition (P2).
  - **Formula**: $P = \frac{TP}{TP + FP}$.

- **Recall (Pair-wise)**
  - **Definition**: Among all sample pairs considered in the same group by the true partition (P2), the proportion also considered in the same group by the predicted partition (P1).
  - **Formula**: $R = \frac{TP}{TP + FN}$.

- **FMI (Fowlkes-Mallows Index)**
  - **Definition**: Geometric mean of Precision and Recall.
  - **Principle**: $FMI = \sqrt{\frac{TP}{TP+FP} \times \frac{TP}{TP+FN}}$.
  - **Range**: `[0, 1]`.
  - **Applicable**: When both Precision and Recall are important.

#### 1.2 Information-Theoretic
*Focuses on the amount of information (entropy) shared between two partitions.*

- **AMI (Adjusted Mutual Information)**
  - **Definition**: Mutual Information corrected for chance.
  - **Principle**: Computes shared information between two partitions based on entropy, and subtracts the random expectation.
  - **Range**: `[0, 1]`. 1 means perfect agreement, 0 means random.
  - **Advantages**:
    - More robust when the number of clusters is very large (even close to the sample size).
    - Captures complex non-linear relationships.
  - **Applicable**: Small samples, many clusters (Large K) scenarios.

- **NMI (Normalized Mutual Information)**
  - **Definition**: Normalized Mutual Information.
  - **Principle**: $NMI = \frac{MI(U, V)}{\sqrt{H(U) \cdot H(V)}}$ (geometric mean) or $\frac{2 \cdot MI}{H(U) + H(V)}$ (arithmetic mean). `necom` uses the geometric mean.
  - **Range**: `[0, 1]`.
  - **Disadvantages**: Not corrected for chance.
  - **Applicable**: Scenarios with balanced cluster size distributions.

- **MI (Mutual Information)**
  - **Definition**: Mutual information between two partitions.
  - **Principle**: $MI(U, V) = \sum \sum P(u,v) \log \frac{P(u,v)}{P(u)P(v)}$.
  - **Range**: `[0, +∞)`.
  - **Disadvantages**: Difficult to interpret directly; strongly affected by partition entropy. Usually used as an intermediate step for AMI/NMI.

- **Homogeneity**
  - **Definition**: Does each cluster contain only members of a single class? (Similar to Precision, requiring clusters to be "pure.")
  - **Principle**: Based on conditional entropy $H(C|K)$. If $H(C|K)=0$, Homogeneity is 1.
  - **Range**: `[0, 1]`.

- **Completeness**
  - **Definition**: Are all members of a class assigned to the same cluster? (Similar to Recall, requiring clusters to be "complete.")
  - **Principle**: Based on conditional entropy $H(K|C)$. If $H(K|C)=0$, Completeness is 1.
  - **Range**: `[0, 1]`.

- **V-Measure**
  - **Definition**: Harmonic mean of Homogeneity and Completeness.
  - **Range**: `[0, 1]`.
  - **Disadvantages**: Not corrected for chance. With small samples or many clusters, scores tend to be high.
  - **Applicable**: When you need to analyze the source of clustering error (over-fragmentation vs. over-mixing).

### 2. Internal Validity
*Used to evaluate the quality of a clustering result itself (compactness and separation) without Ground Truth.*

#### 2.1 Distance-Based
*Requires a distance matrix (`--matrix`) or a phylogenetic tree (`--tree`).*

- **Silhouette Coefficient**
  - **Principle**: For each sample $i$, compute its average distance to samples in the same cluster $a(i)$ and its average distance to the nearest other cluster $b(i)$. $s(i) = (b - a) / \max(a, b)$.
  - **Range**: `[-1, 1]`.
    - Near 1: sample is well clustered (close to same cluster, far from others).
    - 0: sample lies on a cluster boundary.
    - Negative: sample may be mis-clustered.
  - **Advantages**: Intuitive; balances cohesion and separation.
  - **Disadvantages**: High computational complexity ($O(N^2)$); requires optimization for large-scale data.

- **Dunn Index**
  - **Principle**: Ratio of the minimum between-cluster distance to the maximum within-cluster diameter.
  - **Range**: `[0, +∞)`. **Larger is better**.
  - **Advantages**: Simple and intuitive.
  - **Disadvantages**: Extremely sensitive to noise (because it relies on min/max).

- **C-Index**
  - **Principle**: Compares the sum of within-cluster distances with the sum of the smallest $N_W$ distances in the entire dataset ($N_W$ is the number of within-cluster pairs).
  - **Range**: `[0, 1]`. **Smaller is better**.
  - **Disadvantages**: High computational complexity ($O(N^2 \log N)$), requiring sorting of all pairwise distances.

- **Hubert's Gamma**
  - **Principle**: Correlation between the distance matrix and a binary clustering matrix (0 = same cluster, 1 = different clusters).
  - **Range**: `[-1, 1]`. **Larger is better** (note that in the definition Y=1 means different clusters; confirm sign direction for the specific implementation; in `necom` larger means better discriminability).

- **Kendall's Tau**
  - **Principle**: Rank correlation coefficient between the distance matrix and the clustering matrix.
  - **Range**: `[-1, 1]`. **Larger is better**.

#### 2.2 Coordinate-Based
*Requires a coordinate matrix (`--coords`). Suitable for Euclidean-space data.*

- **Davies-Bouldin Index (DBI)**
  - **Principle**: Computes the "similarity" of each cluster pair (within-cluster dispersion sum / centroid distance), then takes the mean of each cluster's worst (largest) similarity.
  - **Range**: `[0, +∞)`. **Smaller is better**.
  - **Advantages**: Faster than Silhouette.
  - **Applicable**: Evaluating centroid-based clustering algorithms.

- **Calinski-Harabasz Index (CH)**
  - **Principle**: Ratio of between-cluster dispersion (BGSS) to within-cluster dispersion (WGSS).
  - **Range**: `[0, +∞)`. **Larger is better**.
  - **Advantages**: Fast to compute.

- **PBM Index**
  - **Principle**: Composite index based on total dispersion, within-cluster dispersion, and maximum centroid distance.
  - **Range**: `[0, +∞)`. **Larger is better**.

- **Ball-Hall Index**
  - **Principle**: Mean of average within-cluster dispersions.
  - **Range**: `[0, +∞)`. **Smaller is better** (more compact).

- **Xie-Beni Index**
  - **Principle**: Ratio of within-cluster compactness to between-cluster separation (minimum centroid distance).
  - **Range**: `[0, +∞)`. **Smaller is better**.

- **Wemmert-Gancarski Index**
  - **Principle**: Compactness index based on relative distances (distance to own centroid / distance to nearest other centroid).
  - **Range**: `[0, 1]`. **Larger is better**.

## Metric Selection Guide

| Scenario | Recommended Metrics | Rationale |
| :--- | :--- | :--- |
| **With Ground Truth (general)** | ARI, AMI | Corrected for chance; reliable. |
| **With Ground Truth (purity)** | V-Measure | Can inspect Homogeneity (purity) and Completeness (coverage) separately. |
| **With Ground Truth (exact matching)** | Jaccard, F1/FMI | Focus on overlap of specific clusters or pairs (rather than overall distribution). |
| **Without Ground Truth (distance)** | Silhouette | Intuitively reflects geometric quality, balancing cohesion and separation. |
| **Without Ground Truth (distance correlation)** | Gamma, Tau | Evaluate the correlation between clustering structure and the original distance matrix. |
| **Without Ground Truth (coordinates)** | Davies-Bouldin, CH | High computational efficiency, suitable for large-scale data. |
| **Without Ground Truth (coordinate compactness)** | PBM, Xie-Beni | Impose stricter penalties on cluster compactness. |
| **Very large number of clusters** | AMI | More stable than ARI. |

## See Also

* [`necom clust`](clust.md) for command overview, supported algorithms, and partition/matrix/coordinate format conventions.
* [`docs/help/eval/partition.md`](help/eval/partition.md) for the command-line help text, including available options and usage examples.
