# necom clust

## Overview

The `necom clust` module provides a collection of clustering algorithms for sequences, genomic features, and general data. These tools are designed to handle the distance matrices, similarity networks, and feature vectors commonly encountered in bioinformatics.

Commands are divided into two categories by input data type (consistent with `necom clust --help`):
1.  **Tree**: Build phylogenetic or hierarchical structures from a distance matrix (`hier`, `nj`, `upgma`).
2.  **Flat**: Generate groups directly from pairwise relations, distances, or similarities (`cc`, `dbscan`, `k-medoids`, `mcl`).

Flat partitions can also be derived from an existing tree using the separate `necom cut` command (see [`docs/cut.md`](cut.md)).

## Algorithm List

### MCL (Markov Cluster Algorithm)

- **Principle**: Simulates random walks on a graph, alternating between "expansion" and "inflation" operations. This concentrates flow within strongly connected regions and causes flow in weakly connected regions to fade, naturally separating out modules.
- **Command**: `necom clust mcl`
- **Characteristics**: Graph clustering based on flow simulation.
- **Use cases**: **Biological networks** (e.g., SSN), protein family detection, module discovery.
- **Advantages**: Robust to noise; handles complex network structures.
- **Input**: Pairwise similarities `.tsv` (higher is better).
- **Output**: `cluster` (default) or `pair` format, controlled by `--format`.
- **Defaults**: `--inflation 2.0`.
- **Note**: `--max-iter` must be greater than 0.

### Connected Components (CC)

- **Principle**: A fundamental graph-theory concept that finds all sets of mutually reachable nodes. Edge weights are ignored; only connectivity matters.
- **Command**: `necom clust cc`
- **Characteristics**: The most basic connected component clustering.
- **Use cases**: Fast deduplication at very high similarity thresholds.
- **Advantages**: Extremely fast (linear complexity).
- **Input**: Pairwise relations in TSV format (`name1  name2  weight`); the weight column is ignored.
- **Output**: `cluster` (default) or `pair` format, controlled by `--format`.

### K-Medoids

- **Principle**: Iterative optimization similar to K-Means, but the center (medoid) must be an actual sample from the dataset. Centers are updated by minimizing the sum of dissimilarities to the nearest center.
- **Command**: `necom clust k-medoids` (alias `km`)
- **Characteristics**: Like K-Means, but centers must be actual samples (medoids).
- **Use cases**: Noise-resistant scenarios, or when only a **distance matrix** (non-Euclidean space) is available.
- **Advantages**: Robust to outliers; interpretable results because centers are real samples.
- **Input**: Pairwise distances `.tsv` (lower is better).
- **Output**: `cluster` (default) or `pair` format, controlled by `--format`.
- **Note**: `--k` must not exceed the number of samples.

### DBSCAN

- **Principle**: Density-based clustering. Starting from any point, if the number of points within its $\epsilon$ neighborhood exceeds `min_points`, it becomes a core point and expands a cluster; regions with insufficient density are treated as noise.
- **Command**: `necom clust dbscan`
- **Characteristics**: Density-based clustering that requires specifying neighborhood radius `eps` and minimum point count `min_points`.
- **Use cases**: **Non-convex** cluster shapes, uneven density distributions, **outlier detection**.
- **Advantages**: Does not require specifying the number of clusters K; identifies noise.
- **Input**: Pairwise distances `.tsv` (lower is better).
- **Output**: `cluster` (one cluster per line, first element is the representative) or `pair` (representative–member pairs).
- **Defaults**: `--eps 0.05`, `--min-points 4`.
- **Note on `--min-points`**: By default (`--same 0.0`), the neighborhood count includes the point itself because self-distance is 0 and is always <= `eps`. If `--same` is set to a value greater than `eps`, the point is not counted as its own neighbor and may fail to become a core point.
- **Unimplemented options**: Parameter scanning and scoring such as `--scan`, `--opt-eps`, `--min-pct` are not yet implemented; planning details are in [`notes/design/dbscan-planned.md`](../notes/design/dbscan-planned.md). They may be provided later as subcommands of `necom clust dbscan` or standalone scripts.

### UPGMA

- **Principle**: Unweighted Pair Group Method with Arithmetic Mean. A bottom-up hierarchical clustering that repeatedly merges the two closest clusters; distances between the new cluster and all others are computed as arithmetic averages of all member-to-member distances. Assumes a constant evolutionary rate (molecular clock).
- **Command**: `necom clust upgma`
- **Characteristics**: Hierarchical clustering (average linkage) that outputs a **rooted tree**.
- **Use cases**: Phylogenetic analysis assuming a **molecular clock** (ultrametric).
- **Advantages**: Produces a hierarchical structure with branch heights carrying clear distance meaning.
- **Input**: PHYLIP distance matrix (strict or relaxed).
- **Output**: Newick tree.
- **Note**: For input that violates the ultrametric assumption, branch lengths are clamped to 0 so the output remains a valid Newick tree.

### NJ (Neighbor-Joining)

- **Principle**: Neighbor-Joining. Iteratively merges the pair of nodes with the smallest net divergence by minimizing total tree length (based on the Q-matrix-corrected distances). Does not assume a molecular clock and allows different evolutionary rates across branches.
- **Command**: `necom clust nj`
- **Characteristics**: Distance-matrix tree-building algorithm that outputs a **midpoint-rooted Newick tree**.
- **Use cases**: General additive distances (no molecular-clock assumption); evolutionary tree construction.
- **Advantages**: Fast; robust to different evolutionary rates.
- **Input**: PHYLIP distance matrix (strict or relaxed).
- **Output**: Midpoint-rooted Newick tree.
- **Note**: For non-additive distances, negative branch lengths are clamped to 0 so the output remains a valid Newick tree.

### Hierarchical Clustering

- **Principle**: A general bottom-up (agglomerative) clustering framework. Clusters are merged according to different linkage criteria (e.g., Ward minimum variance, Complete maximum distance), building a complete dendrogram hierarchy.
- **Command**: `necom clust hier` (alias `hclust`)
- **Characteristics**: General hierarchical clustering supporting `single`, `complete`, `average`, `weighted`, `centroid`, `median`, `ward`.
- **Default method**: `ward` (use `--method` to select other linkage criteria).
- **Implementation status**: Implemented with $O(N^2)$ NN-chain optimization for reducible methods (`single`, `complete`, `average`, `weighted`, `ward`); `centroid` and `median` fall back to the primitive $O(N^3)$ implementation because they do not satisfy the reducibility property.
- **Use cases**: General hierarchical clustering analysis; combined with `necom cut` for flexible groupings at different granularities.
- **Advantages**: Supports multiple linkage methods; the default NN-chain implementation achieves $O(N^2)$ time for reducible methods.
- **Input**: PHYLIP distance matrix (strict or relaxed).
- **Output**: Newick tree.
- **Details**: [Hierarchical Clustering Details](#hierarchical-clustering-details)

## Hierarchical Clustering Details

`necom clust hier` (alias `hclust`) provides general hierarchical clustering (dendrogram) generation, supporting `single`, `complete`, `average`, `weighted`, `centroid`, `median`, and `ward` methods, outputting Newick format for downstream `necom cut`.

### Background and Positioning

- **Module**: `clust`, alongside `k-medoids`, `mcl`, etc.
- **Goal**: Statistically meaningful dendrograms (merge heights express the cost of the linkage criterion), without enforcing "evolution/molecular-clock" semantics.
- **Synergy with existing necom capabilities**:
  - Tree building: `clust upgma` (rooted, ultrametric) and `clust nj` (additive, unrooted) already exist.
  - Cutting: tree-cut grouping via `necom cut`.
  - Evaluation: `necom eval partition --matrix` / `--tree` / `--coords` (currently available); `necom eval tree` not yet implemented.

### Relationship to UPGMA/NJ

- Commonalities: All take a distance matrix as input and output a tree-like structure; all can be combined with `necom cut` to obtain flat groupings.
- Relationship to UPGMA:
  - R `hclust(method="average")` is equivalent to "average linkage"; UPGMA is a specialized version under the "ultrametric (molecular clock)" assumption, producing a rooted, strictly ultrametric tree whose branch lengths have "time/evolution" meaning.
  - Conclusion: The linkage updates are identical, but the semantics differ; UPGMA leans toward phylogenetic scenarios, while `clust hier` leans toward statistical clustering.
- Relationship to NJ:
  - NJ (Neighbor-Joining) minimizes total tree length via the Q matrix, producing an "additive minimum-length tree" that does not belong to the linkage-update paradigm and outputs a **midpoint-rooted Newick tree**.
  - For general additive distances, NJ is more robust than UPGMA; if the distances are ultrametric, UPGMA/hclust-average and NJ usually agree topologically (unrooted view).

### Methods and Algorithm Essentials

- `single/complete/average`: Standard linkage updates (Lance–Williams framework); merge height is the distance/cost corresponding to the linkage criterion.
- `ward`:
  - Concept: Minimizes the increase in within-cluster sum of squares (total within-group variance, SSE); commonly used and robust.
  - Update (squared-distance version, where n is cluster size):
    - Let the squared distance between merged cluster `u∪v` and a third cluster `w` be:
    - `d(u∪v,w)^2 = [ (n_u+n_w) d(u,w)^2 + (n_v+n_w) d(v,w)^2 − n_w d(u,v)^2 ] / (n_u+n_v+n_w)`
  - If the input is non-squared distances: square them for the update, and take the square root or use the SSE-increment definition for merge heights when outputting.
  - Distance prerequisite: Theoretically requires Euclidean or near-Euclidean distances; usable on general biological distances, but the statistical interpretation of "variance minimization" becomes weaker.

### Output and Conventions

- Outputs Newick dendrogram:
  - Merges are emitted in the order they are performed by the linkage algorithm (merge-order output). The internal node IDs increase with the merge sequence, matching the convention used by R `hclust` and SciPy `linkage`.
  - Internal node height is half the merge distance (`height = distance / 2`), and branch length from child to parent is `parent_height - child_height`.
  - Therefore the output is ultrametric-like: all leaves under the same internal node have equal total distance to that node.
  - Branch lengths express merge heights (linkage cost or SSE increment with appropriate unit handling).
  - Strict ultrametricity is not guaranteed (unless the data satisfy the corresponding conditions), but the output satisfies the requirements of `necom cut --height`.
- Numeric format: branch lengths are emitted with Rust's default float formatting. For a fixed-width, six-decimal view consistent with `nwk distance`, post-process the tree or use `nwk distance` on the resulting branch lengths.

### Notes

- `clust hier` only accepts **distance matrices** (smaller values mean higher similarity). Similarity matrices must be converted first, e.g., with `necom mat transform`.
- `ward` updates use squared distances internally; output branch lengths are expressed in the original distance units, so you do not need to square the input.
- `ward` theoretically assumes Euclidean or near-Euclidean distances; on general biological distances the statistical interpretation of "minimum variance" is weaker.
- `centroid` and `median` linkage may produce non-monotonic merge heights (inversions); this is an algorithmic characteristic of these methods.
- For reducible methods (`single`, `complete`, `average`, `weighted`, `ward`), the default NN-chain implementation produces the same dendrogram as the primitive $O(N^3)$ algorithm, but the concrete merge order may differ. Consequently, the Newick string may have a different internal node ordering than `clust upgma` for the same method, even though the underlying clustering is equivalent.
- Ties in nearest-neighbor selection are broken deterministically by cluster index; because `hier` operates on the indexed distance matrix, this is independent of sample name alphabetical order.

### Recommended Hier Workflow

- Generate tree:
  - Near-molecular-clock/ultrametric scenarios: `clust upgma` outputs a rooted ultrametric tree.
  - General additive-distance scenarios: `clust nj`.
  - General hierarchical analysis or when `ward` is needed: `clust hier --method ward`.
- Cut and evaluate:
  - Cut: `necom cut --height H` or TreeCluster-style thresholds/constraints.
  - Internal evaluation (no Ground Truth): `necom eval partition --matrix ...` (Silhouette) (currently available); `necom eval tree` not yet implemented.
  - External evaluation (with Ground Truth): `necom eval partition` (ARI/AMI/V-Measure).

## Evaluation

Partition evaluation has moved to [`necom eval partition`](eval.md). See [`docs/eval.md`](eval.md) for the overview and [`docs/eval-partition.md`](eval-partition.md) for detailed metric definitions.

## Planned

GMM, HDBSCAN, Louvain/Leiden and other algorithms are on the roadmap. Algorithms considered but not adopted (K-Means, Spectral, OPTICS, BIRCH, etc.) are documented in [`notes/design/clust-impl.md`](../notes/design/clust-impl.md).

### Bootstrap Support for Hierarchical Clustering [Planned]

`necom clust boot` will compute multiscale-bootstrap BP/AU/SI support values (pvclust-style) for each internal node of a hierarchical clustering tree, quantifying the stability of clusters under feature resampling. Design details are in [`notes/design/clust-boot.md`](../notes/design/clust-boot.md).

### GMM (Gaussian Mixture Models) [Planned]

Motivation for introducing GMM:
- **Soft Clustering**: Unlike the hard assignment of K-means, GMM gives the probability that a sample belongs to each cluster, suitable for fuzzy biological classifications (e.g., subspecies, gene-family transitional states).
- **Non-spherical clusters**: Models different shapes and sizes through covariance matrices (K-means assumes equal-variance spherical clusters).
- **Generative model**: Can be used for density estimation and outlier detection.

**Planned interface**:
```bash
# GMM clustering from CSV/TSV vector input
necom clust gmm input.tsv --k 5 --cov full > clusters.tsv

# Output includes: ID, Cluster, PosteriorProb
```

### Model Selection

How to determine the number of clusters (K) or the best model complexity?

- **BIC (Bayesian Information Criterion)** [Planned]:
  - In GMM, BIC trades off log-likelihood (goodness of fit) against the number of parameters (complexity).
  - `necom` could provide `clust gmm --scan-k 2..20`, automatically computing and outputting a BIC curve to help users choose the best K (usually the BIC minimum or elbow).
- **Silhouette / Calinski-Harabasz** [Partially supported]: Geometry-based evaluation metrics suitable for K-means or general distance clustering (`eval partition` already supports distance-matrix Silhouette; tree-based Silhouette is planned for `necom eval tree` [Planned]).

## Large-Scale Data Strategy

For large-scale data with $N > 20,000$, the memory ($O(N^2)$) and computation ($O(N^2)$) costs of fully connected hierarchical clustering increase sharply.

**Memory estimate (f32 condensed matrix)**:
- **1 GiB**: ~23,000 points
- **10 GiB**: ~73,000 points
- **32 GiB**: ~130,000 points
- **64 GiB**: ~185,000 points

**Conclusion**: Even on a high-end server with 64 GiB memory, processing $N=200k$ is near the limit.

**Recommended strategy**: Use a "two-step" approach combining fast clustering with careful tree building.
1.  **Pre-clustering/compression**: Use linear or near-linear algorithms (e.g., `necom clust k-medoids`, `necom clust mcl`, or external tools such as `mmseqs2`) to compress data into $K$ representative points ($K \approx 5000 \sim 10000$).
2.  **Hierarchical clustering**: Extract the distance matrix among representative points and run `necom clust hier` to build the backbone tree.

**Workflow example**:
```bash
# 1. Fast clustering to select representatives (k=5000)
necom clust k-medoids all_data.tsv --k 5000 --format pair > clusters.tsv

# 2. Extract representative list (Unix `cut`, not `necom cut`)
cut -f1 clusters.tsv | sort -u > representatives.list

# 3. Extract sub-matrix for representatives
necom mat subset all_data.tsv representatives.list -o sub_matrix.phy

# 4. Build tree on representatives
necom clust hier sub_matrix.phy --method ward > backbone.nwk
```

## Recommended Workflows

### Scenario A: Protein Family Mining (Graph-based)

```bash
# 1. Build sequence-alignment network (e.g., mmseqs/blast -> pair.tsv)
# 2. MCL clustering
necom clust mcl pairs.tsv --inflation 2.0 > families.tsv
```

### Scenario B: Hierarchical Clustering Parameter Scanning and Evaluation Workflow

Combine `necom cut` scanning with `eval partition` batch evaluation to find the best cutting threshold.

```bash
# 1. Generate hierarchical clustering tree
necom clust hier matrix.phy --method ward > tree.nwk

# 2. Scan thresholds, save to a file, and evaluate internal metrics (Silhouette)
# necom cut outputs a long table in scan mode; write it to a file for eval partition
necom cut tree.nwk --height 1.0 --scan 0,1.0,0.05 > partitions.tsv
necom eval partition partitions.tsv --input-format long --matrix matrix.phy > evaluation.tsv

# 3. Analyze evaluation.tsv to choose the best threshold (e.g., maximum Silhouette)
# Assume the best threshold is 0.45
necom cut tree.nwk --height 0.45 > final_clusters.tsv
```

For input/output format conventions used by `necom clust` and other commands, see [`docs/formats.md`](formats.md).
