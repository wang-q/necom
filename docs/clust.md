# necom clust

## Overview

The `necom clust` module provides a collection of clustering algorithms for sequences, genomic
features, and general data. These tools are designed to handle the distance matrices, similarity
networks, and feature vectors commonly encountered in bioinformatics.

Commands are divided into two categories by input data type (consistent with `necom clust --help`):

1. **Tree**: Build phylogenetic or hierarchical structures from a distance matrix (`hier`, `nj`,
   `upgma`).
2. **Flat**: Generate groups directly from pairwise relations, distances, or similarities (`cc`,
   `dbscan`, `k-medoids`, `mcl`). DBSCAN parameter scanning is available as `scan-dbscan`.

Flat partitions can also be derived from an existing tree using the separate `necom cut`
command (see [`docs/cut.md`](cut.md)).

## Input and Output Conventions

### Input by command type

- **Tree-building commands** (`hier`, `nj`, `upgma`): accept a PHYLIP distance matrix (strict or
  relaxed). Smaller values mean higher similarity.
- **Flat clustering commands**:
    - `mcl` and `cc`: accept pairwise **similarities** in TSV format (`name1\tname2\tscore`);
      higher is better.
    - `dbscan` and `k-medoids`: accept pairwise **distances** in TSV format (`name1\tname2\tdist`);
      lower is better.

Similarity matrices must be converted to distances before `dbscan` or `k-medoids`, e.g., with
`necom mat transform`.

### Output formats

Most flat-clustering commands support `--format`:

- `cluster` (default): one cluster per line, first element is the representative, remaining
  elements are members. Noise points in DBSCAN are emitted as single-member clusters.
- `pair`: one `representative\tmember` pair per line.

Tree-building commands always output a Newick tree.

## Algorithm List

### MCL (Markov Cluster Algorithm)

- **Principle**: Simulates random walks on a graph, alternating between "expansion" and "inflation"
  operations. This concentrates flow within strongly connected regions and causes flow in weakly
  connected regions to fade, naturally separating out modules.
- **Command**: `necom clust mcl`
- **Characteristics**: Graph clustering based on flow simulation.
- **Use cases**: **Biological networks** (e.g., SSN), protein family detection, module discovery.
- **Advantages**: Robust to noise; handles complex network structures.
- **Input**: Pairwise similarities `.tsv` (higher is better).
- **Output**: `cluster` (default) or `pair` format, controlled by `--format`.
- **Defaults**: `--inflation 2.0`, `--prune 1e-5`, `--max-iter 100`.
- **Note**: `--max-iter` must be greater than 0.

### Connected Components (CC)

- **Principle**: A fundamental graph-theory concept that finds all sets of mutually reachable nodes.
  Edge weights are ignored; only connectivity matters.
- **Command**: `necom clust cc`
- **Characteristics**: The most basic connected component clustering.
- **Use cases**: Fast deduplication at very high similarity thresholds.
- **Advantages**: Extremely fast (linear complexity).
- **Input**: Pairwise relations in TSV format (`name1\tname2\t[weight]`); the weight column is
  ignored.
- **Output**: `cluster` (default) or `pair` format, controlled by `--format`.

### K-Medoids

- **Principle**: Iterative optimization similar to K-Means, but the center (medoid) must be an
  actual sample from the dataset. Centers are updated by minimizing the sum of dissimilarities to
  the nearest center.
- **Command**: `necom clust k-medoids` (alias `km`)
- **Characteristics**: Like K-Means, but centers must be actual samples (medoids).
- **Use cases**: Noise-resistant scenarios, or when only a **distance matrix** (non-Euclidean space)
  is available.
- **Advantages**: Robust to outliers; interpretable results because centers are real samples.
- **Input**: Pairwise distances `.tsv` (lower is better).
- **Output**: `cluster` (default) or `pair` format, controlled by `--format`.
- **Defaults**: `--runs 10`, `--max-iter 100`.
- **Note**: `--k` must not exceed the number of samples.
- **Note**: If a cluster becomes empty during iteration, it is omitted from the output, so the
  final number of clusters may be less than `k`.

### DBSCAN

- **Principle**: Density-based clustering. Starting from any point, if the number of points within
  its $\epsilon$ neighborhood exceeds `min_points`, it becomes a core point and expands a cluster;
  regions with insufficient density are treated as noise.
- **Command**: `necom clust dbscan`
- **Characteristics**: Density-based clustering that requires specifying neighborhood radius `eps`
  and minimum point count `min_points`.
- **Use cases**: **Non-convex** cluster shapes, uneven density distributions, **outlier detection**.
- **Advantages**: Does not require specifying the number of clusters K; identifies noise.
- **Input**: Pairwise distances `.tsv` (lower is better).
- **Output**: `cluster` (one cluster per line, first element is the representative) or
  `pair` (representative–member pairs). Noise points are emitted as single-member clusters.
- **Defaults**: `--eps 0.05`, `--min-points 4`.
- **Note on `--min-points`**: By default (`--same 0.0`), the neighborhood count includes the point
  itself because self-distance is 0 and is always <= `eps`. If `--same` is set to a value greater
  than `eps`, the point is not counted as its own neighbor and may fail to become a core point.
- **Representative selection**: By default (`--rep medoid`), the representative of each cluster
  is its medoid (the point with the smallest average distance to other cluster members). Use
  `--rep first` to use the first-discovered point instead. Noise points are emitted as single-member
  clusters where the representative is the point itself.
- **`--min-pct`**: Alternative to `--min-points`; specify `min_points` as a fraction of the total
  number of samples. The effective value is `ceil(P * n_samples)`, range `(0, 1]`. Mutually
  exclusive with `--min-points`.
- **Parameter scanning**: To scan `eps` and compare internal metrics, use
  `necom clust scan-dbscan` (see below).

### DBSCAN Scan (`scan-dbscan`)

- **Principle**: Run DBSCAN repeatedly over a range of `eps` values while keeping `min_points`
  fixed. For each `eps`, report the number of clusters, the number of noise points, and two
  internal validity indices (Silhouette and Davies-Bouldin). Optionally, select the best `eps`
  according to a criterion and output the corresponding partition.
- **Command**: `necom clust scan-dbscan`
- **Characteristics**: Parameter exploration for DBSCAN without ground truth.
- **Use cases**: Choose an appropriate `eps` when the distance scale of the data is unknown.
- **Input**: Pairwise distances `.tsv` (lower is better).
- **Output**:
    - Without `--opt-eps`: a TSV table with columns `Epsilon`, `Clusters`, `Noise`, `Silhouette`,
      `DBIndex`.
    - With `--opt-eps`: a clustering partition in `cluster` or `pair` format, using the same
      conventions as `necom clust dbscan`.
- **Required**: `--scan <start,end,step>`.
- **Options**:
    - `--min-points <N>` or `--min-pct <P>`: same semantics as `necom clust dbscan`.
    - `--opt-eps {silhouette|max-clusters|min-noise}`: select the best `eps` and output its
      partition.
        - `silhouette`: maximize Silhouette score.
        - `max-clusters`: maximize the number of non-noise clusters.
        - `min-noise`: minimize the number of noise points.
        - Ties are resolved by choosing the smaller `eps`.
- **Metrics**: Silhouette and Davies-Bouldin use the distance-matrix definitions described in
  [`docs/eval-partition.md`](eval-partition.md). Noise points are treated as singleton clusters
  when computing metrics.
- **Performance**: Scanning costs `steps × O(N²)`; reduce the range or step count for large
  matrices.
- **Note**: The explicit `end` value of `--scan` is always included, even if the step size does not
  divide the interval evenly.

### UPGMA

- **Principle**: Unweighted Pair Group Method with Arithmetic Mean. A bottom-up hierarchical
  clustering that repeatedly merges the two closest clusters; distances between the new cluster
  and all others are computed as arithmetic averages of all member-to-member distances. Assumes a
  constant evolutionary rate (molecular clock).
- **Command**: `necom clust upgma`
- **Characteristics**: Hierarchical clustering (average linkage) that outputs a **rooted tree**.
- **Use cases**: Phylogenetic analysis assuming a **molecular clock** (ultrametric).
- **Advantages**: Produces a hierarchical structure with branch heights carrying clear distance
  meaning.
- **Input**: PHYLIP distance matrix (strict or relaxed).
- **Output**: Newick tree.
- **Note**: For input that violates the ultrametric assumption, branch lengths are clamped to 0 so
  the output remains a valid Newick tree.

### NJ (Neighbor-Joining)

- **Principle**: Neighbor-Joining. Iteratively merges the pair of nodes with the smallest net
  divergence by minimizing total tree length (based on the Q-matrix-corrected distances). Does not
  assume a molecular clock and allows different evolutionary rates across branches.
- **Command**: `necom clust nj`
- **Characteristics**: Distance-matrix tree-building algorithm that outputs a Newick tree rooted at
  the midpoint of the final edge.
- **Use cases**: General additive distances (no molecular-clock assumption); evolutionary tree
  construction.
- **Advantages**: Fast; robust to different evolutionary rates.
- **Input**: PHYLIP distance matrix (strict or relaxed).
- **Output**: Newick tree rooted at the midpoint of the final edge.
- **Note**: For non-additive distances, negative branch lengths are clamped to 0 so the output
  remains a valid Newick tree.

### Hierarchical Clustering

- **Principle**: A general bottom-up (agglomerative) clustering framework. Clusters are merged
  according to different linkage criteria (e.g., Ward minimum variance, Complete maximum distance),
  building a complete dendrogram hierarchy.
- **Command**: `necom clust hier` (alias `hclust`)
- **Characteristics**: General hierarchical clustering supporting `single`, `complete`, `average`,
  `weighted`, `centroid`, `median`, `ward`.
- **Default method**: `ward` (use `--method` to select other linkage criteria).
- **Implementation status**: Implemented with $O(N^2)$ NN-chain optimization for reducible
  methods (`single`, `complete`, `average`, `weighted`, `ward`); `centroid` and `median` fall back
  to the primitive $O(N^3)$ implementation because they do not satisfy the reducibility property.
- **Use cases**: General hierarchical clustering analysis; combined with `necom cut` for flexible
  groupings at different granularities.
- **Advantages**: Supports multiple linkage methods; the default NN-chain implementation achieves
  $O(N^2)$ time for reducible methods.
- **Input**: PHYLIP distance matrix (strict or relaxed).
- **Output**: Newick tree.
- **Details**: [Hierarchical Clustering Details](#hierarchical-clustering-details)

## Hierarchical Clustering Details

`necom clust hier` (alias `hclust`) provides general hierarchical clustering (dendrogram)
generation, supporting `single`, `complete`, `average`, `weighted`, `centroid`, `median`, and
`ward` methods, outputting Newick format for downstream `necom cut`.

### Background and Positioning

- **Module**: `clust`, alongside `k-medoids`, `mcl`, etc.
- **Goal**: Statistically meaningful dendrograms (merge heights express the cost of the linkage
  criterion), without enforcing "evolution/molecular-clock" semantics.
- **Synergy with existing necom capabilities**:
    - Tree building: `clust upgma` (rooted, ultrametric) and `clust nj` (additive, rooted at the
      midpoint of the final edge) already exist.
    - Cutting: tree-cut grouping via `necom cut`.
    - Evaluation: `necom eval partition --matrix` / `--tree` / `--coords` (currently available);
      `necom eval tree` not yet implemented.

### Relationship to UPGMA/NJ

- Commonalities: All take a distance matrix as input and output a tree-like structure; all can be
  combined with `necom cut` to obtain flat groupings.
- Relationship to UPGMA:
    - R `hclust(method="average")` is equivalent to "average linkage"; UPGMA is a specialized
      version under the "ultrametric (molecular clock)" assumption, producing a rooted, strictly
      ultrametric tree whose branch lengths have "time/evolution" meaning.
    - Conclusion: The linkage updates are identical, but the semantics differ; UPGMA leans toward
      phylogenetic scenarios, while `clust hier` leans toward statistical clustering.
- Relationship to NJ:
    - NJ (Neighbor-Joining) minimizes total tree length via the Q matrix, producing an "additive
      minimum-length tree" that does not belong to the linkage-update paradigm and outputs a Newick
      tree rooted at the midpoint of the final edge.
    - For general additive distances, NJ is more robust than UPGMA; if the distances are
      ultrametric, UPGMA/hclust-average and NJ usually agree topologically (unrooted view).

### Methods and Algorithm Essentials

- `single/complete/average`: Standard linkage updates (Lance–Williams framework); merge height is
  the distance/cost corresponding to the linkage criterion.
- `ward`:
    - Concept: Minimizes the increase in within-cluster sum of squares (total within-group variance,
      SSE); commonly used and robust.
    - Update (squared-distance version, where n is cluster size):
        - Let the squared distance between merged cluster `u∪v` and a third cluster `k` be:
        - `d(u∪v,k)^2 = [ (n_u+n_k) d(u,k)^2 + (n_v+n_k) d(v,k)^2 − n_k d(u,v)^2 ] / (n_u+n_v+n_k)`
    - If the input is non-squared distances: square them for the update, and take the square root
      or use the SSE-increment definition for merge heights when outputting.
    - Distance prerequisite: Theoretically requires Euclidean or near-Euclidean distances; usable on
      general biological distances, but the statistical interpretation of "variance minimization"
      becomes weaker.

### Output and Conventions

- Outputs Newick dendrogram:
    - Merges are emitted in the order they are performed by the linkage algorithm (merge-order
      output). The internal node IDs increase with the merge sequence, matching the convention used
      by R `hclust` and SciPy `linkage`.
    - Internal node height is half the merge distance (`height = distance / 2`), and branch length
      from child to parent is `parent_height - child_height`.
    - For reducible methods (`single`/`complete`/`average`/`weighted`/`ward`), the output is
      ultrametric-like: all leaves under the same internal node have equal total distance to
      that node. `centroid`/`median` may violate this property due to non-monotonic merge
      heights (inversions), even after negative branch lengths are clamped to zero.
    - Branch lengths express merge heights (linkage cost or SSE increment with appropriate unit
      handling).
    - Strict ultrametricity is not guaranteed (unless the data satisfy the corresponding
      conditions), but the output satisfies the requirements of `necom cut simple --height <H>`.
- Numeric format: branch lengths are emitted with Rust's default float formatting. For a
  fixed-width, six-decimal view consistent with `necom nwk distance`, post-process the tree or use
  `necom nwk distance` on the resulting branch lengths.

### Notes

- `clust hier` only accepts **distance matrices** (smaller values mean higher similarity).
  Similarity matrices must be converted first, e.g., with `necom mat transform`.
- `ward`, `centroid`, and `median` updates use squared distances internally; output branch lengths
  are expressed in the original distance units, so you do not need to square the input.
- `ward` theoretically assumes Euclidean or near-Euclidean distances; on general biological
  distances the statistical interpretation of "minimum variance" is weaker.
- `centroid` and `median` linkage may produce non-monotonic merge heights (inversions); this is an
  algorithmic characteristic of these methods.
- For reducible methods (`single`, `complete`, `average`, `weighted`, `ward`), the default NN-chain
  implementation produces the same dendrogram as the primitive $O(N^3)$ algorithm, but the
  concrete merge order may differ. Consequently, the Newick string may have a different internal
  node ordering than `clust upgma` for the same method, even though the underlying clustering is
  equivalent.
- Ties in nearest-neighbor selection are broken deterministically by cluster index; because `hier`
  operates on the indexed distance matrix, this is independent of sample name alphabetical order.

### Recommended Hier Workflow

- Generate tree:
    - Near-molecular-clock/ultrametric scenarios: `clust upgma` outputs a rooted ultrametric tree.
    - General additive-distance scenarios: `clust nj`.
    - General hierarchical analysis or when `ward` is needed: `clust hier --method ward`.
- Cut and evaluate:
    - Cut: `necom cut simple tree.nwk --height H` or TreeCluster-style thresholds/constraints.
    - Internal evaluation (no Ground Truth): `necom eval partition --matrix ...` (Silhouette) (currently
      available); `necom eval tree` not yet implemented.
    - External evaluation (with Ground Truth): `necom eval partition` (ARI/AMI/V-Measure).

## Evaluation

Partition evaluation has moved to [`necom eval partition`](eval.md). See [`docs/eval.md`](eval.md)
for the overview and [`docs/eval-partition.md`](eval-partition.md) for detailed metric definitions.

## Implementation Status

| Command     | Algorithm                | Status    | Notes                                                            |
|-------------|--------------------------|-----------|------------------------------------------------------------------|
| `mcl`       | Markov Cluster Algorithm | Available | Flow simulation on graphs; uses pairwise similarities.           |
| `cc`        | Connected Components     | Available | Fast graph connectivity; ignores edge weights.                   |
| `k-medoids` | K-Medoids                | Available | Medoids are real samples; supports distance matrices.            |
| `dbscan`    | DBSCAN                   | Available | Density-based; outputs representative–member pairs.              |
| `upgma`     | UPGMA                    | Available | Rooted, ultrametric tree; assumes molecular clock.               |
| `nj`        | Neighbor-Joining         | Available | Rooted at midpoint of final edge; no molecular-clock assumption. |
| `hier`      | Hierarchical clustering  | Available | 7 linkage methods; NN-chain optimization for reducible methods.  |
| `gmm`       | Gaussian Mixture Models  | Planned   | Soft clustering with BIC model selection.                        |
| `hdbscan`   | HDBSCAN                  | Planned   | Density-based hierarchical clustering without global `eps`.      |
| *TBD*       | Louvain / Leiden         | Planned   | Modularity-based community detection for large networks.         |

## Algorithm Selection Guide

Choosing the right clustering command depends on the input data and the biological or analytical
question.

- **Need a tree from a distance matrix?**
    - `necom clust upgma` — when the data are expected to be ultrametric (molecular clock).
    - `necom clust nj` — for general additive distances without a molecular-clock assumption.
    - `necom clust hier --method ward` — for statistical hierarchical clustering with flexible
      linkage criteria.
- **Need flat groups from pairwise similarities?**
    - `necom clust mcl` — biological networks and protein families.
    - `necom clust cc` — fast deduplication when connectivity alone matters.
- **Need flat groups from pairwise distances?**
    - `necom clust k-medoids` — when centers must be actual samples or the metric is non-Euclidean.
    - `necom clust dbscan` — for non-convex shapes, uneven densities, or outlier detection.
- **Have feature vectors?**
    - Use `necom eval partition --coords` (via `necom eval`) for geometry-based validation, or wait
      for the planned `necom clust gmm` command.

## Why Some Common Algorithms Are Not Provided

The following classic algorithms are intentionally not included in `necom clust`. They have clear
limitations for the biological sequence and network scales this project targets, and a better
alternative already exists or is planned.

- **K-Means**: Assumes spherical, equal-variance clusters and centroids that are not real samples.
  Use `necom clust k-medoids` instead.
- **Bisecting K-Means**: Inherits K-Means limitations and favors divisive splitting. Use
  `necom clust hier` or `necom clust upgma` for bottom-up trees.
- **Affinity Propagation**: $O(N^2)$ cost makes it impractical for > 10k sequences. Use
  `necom clust k-medoids` for small representative sets, or `necom clust dbscan` / `necom clust mcl`
  for automatic cluster counts.
- **Spectral Clustering**: Building the Laplacian and eigendecomposition is expensive ($O(N^3)$).
  Use `necom clust mcl` on biological networks for similar results with better scalability.
- **Mean Shift**: High complexity and sensitive bandwidth selection. Use `necom clust dbscan` or
  the planned `necom clust gmm`.
- **OPTICS**: Its core idea is better automated by HDBSCAN. Use the planned `necom clust hdbscan`.
- **Biclustering**: Designed for gene-expression matrix sub-blocks, not sample grouping. Use
  specialized tools such as WGCNA when needed.
- **BIRCH**: Relies on Euclidean statistics and restricts cluster shapes. Use external vector tools
  for large-scale vectors, or `necom clust mcl` for large networks.

## Planned

GMM, HDBSCAN, Louvain/Leiden and other algorithms are on the roadmap. Detailed
implementation analysis and algorithms considered but not adopted are documented in
[`notes/design/clust-impl.md`](../notes/design/clust-impl.md).

### Bootstrap Support for Hierarchical Clustering \[Moved to `necom eval`\]

`necom eval boot` will compute multiscale-bootstrap BP/AU/SI support values (pvclust-style) for each
internal node of a hierarchical clustering tree, quantifying the stability of clusters under feature
resampling. Design details are in [`notes/design/eval-boot.md`](../notes/design/eval-boot.md).

### GMM (Gaussian Mixture Models) \[Planned\]

Motivation for introducing GMM:

- **Soft Clustering**: Unlike the hard assignment of K-means, GMM gives the probability that a
  sample belongs to each cluster, suitable for fuzzy biological classifications (e.g., subspecies,
  gene-family transitional states).
- **Non-spherical clusters**: Models different shapes and sizes through covariance matrices (K-means
  assumes equal-variance spherical clusters).
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
    - In GMM, BIC trades off log-likelihood (goodness of fit) against the number of
      parameters (complexity).
    - `necom` could provide `clust gmm --scan-k 2..20`, automatically computing and outputting a
      BIC curve to help users choose the best K (usually the BIC minimum or elbow).
- **Silhouette / Calinski-Harabasz** [Partially supported]: Geometry-based evaluation metrics
  suitable for K-means or general distance clustering (`eval partition` already supports
  distance-matrix Silhouette; tree-based Silhouette is planned for `necom eval tree` [Planned]).

## Large-Scale Data Strategy

For large-scale data with $N > 20,000$, the memory ($O(N^2)$) and computation ($O(N^2)$) costs of
fully connected hierarchical clustering increase sharply.

**Memory estimate (f32 condensed matrix)**:

- **1 GiB**: ~23,000 points
- **10 GiB**: ~73,000 points
- **32 GiB**: ~130,000 points
- **64 GiB**: ~185,000 points

**Implications**: Even on a high-end server with 64 GiB memory, processing $N=200k$ is near the
limit.

**Recommended strategy**: Use a "two-step" approach combining fast clustering with careful tree
building.

1. **Pre-clustering/compression**: Use linear or near-linear algorithms (e.g.,
   `necom clust k-medoids`, `necom clust mcl`, or external tools such as `mmseqs2`) to compress
   data into $K$ representative points ($K \approx 5000 \sim 10000$).
2. **Hierarchical clustering**: Extract the distance matrix among representative points and run
   `necom clust hier` to build the backbone tree.

**Workflow example**:

```bash
# 1. Fast clustering to select representatives (k=5000)
necom clust k-medoids all_distances.tsv --k 5000 --format pair > clusters.tsv

# 2. Extract representative list (Unix `cut`, not `necom cut`)
cut -f1 clusters.tsv | sort -u > representatives.list

# 3. Extract sub-matrix for representatives
necom mat subset all_distances.phy representatives.list -o sub_matrix.phy

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

Combine `necom cut` scanning with `eval partition` batch evaluation to find the best cutting
threshold.

```bash
# 1. Generate hierarchical clustering tree
necom clust hier matrix.phy --method ward > tree.nwk

# 2. Scan thresholds, save to a file, and evaluate internal metrics (Silhouette)
# necom cut scan-simple outputs a long table; write it to a file for eval partition
necom cut scan-simple tree.nwk --height --range 0,1.0,0.05 > partitions.tsv
necom eval partition partitions.tsv --input-format long --matrix matrix.phy > evaluation.tsv

# 3. Analyze evaluation.tsv to choose the best threshold (e.g., maximum Silhouette)
# Assume the best threshold is 0.45
necom cut simple tree.nwk --height 0.45 > final_clusters.tsv
```

For input/output format conventions used by `necom clust` and other commands, see
[`docs/formats.md`](formats.md).

