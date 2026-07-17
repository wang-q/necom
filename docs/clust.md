# necom clust

## Overview

The `necom clust` module provides a collection of clustering algorithms for sequences, genomic features, and general data. These tools are designed to handle the distance matrices, similarity networks, and feature vectors commonly encountered in bioinformatics.

Commands are divided into three categories by input data type (consistent with `necom clust --help`):
1.  **Tree**: Build phylogenetic or hierarchical structures from a distance matrix (`hier`, `nj`, `upgma`).
2.  **Flat**: Generate groups directly from graphs or vectors, or derive groups from an existing tree (`cc`, `cut`, `dbscan`, `k-medoids`, `mcl`).
3.  **Eval**: Assess the quality of clustering partitions (`eval`). See [Evaluation and Analysis](#evaluation-and-analysis) below.

## Algorithm List

### MCL (Markov Cluster Algorithm)

- **Principle**: Simulates random walks on a graph, alternating between "expansion" and "inflation" operations. This concentrates flow within strongly connected regions and causes flow in weakly connected regions to fade, naturally separating out modules.
- **Command**: `necom clust mcl`
- **Characteristics**: Graph clustering based on flow simulation.
- **Use cases**: **Biological networks** (e.g., SSN), protein family detection, module discovery.
- **Advantages**: Robust to noise; handles complex network structures.
- **Input**: Pairwise similarities `.tsv` (higher is better).
- **Output**: `cluster` (default) or `pair` format, controlled by `--format`.
- **CLI options**: `infile`, `--format {cluster|pair}`, `--rep {medoid|first}`, `--same <FLOAT>`, `--missing <FLOAT>`, `--inflation <FLOAT>`, `--prune <FLOAT>`, `--max-iter <N>`, `-o/--outfile`.
  - `--inflation` default: `2.0` (higher values produce tighter, more clusters).
  - `--prune` default: `1e-5` (matrix entries below this are set to zero).
  - `--max-iter` default: `100`.
  - `--rep` default: `medoid` (maximum sum of similarities to other members); `first` uses the alphabetically first member.
  - `--same` default: `1.0`; `--missing` default: `0.0`.

### Connected Components (CC)

- **Principle**: A fundamental graph-theory concept that finds all sets of mutually reachable nodes. Edge weights are ignored; only connectivity matters.
- **Command**: `necom clust cc`
- **Characteristics**: The most basic connected component clustering.
- **Use cases**: Fast deduplication at very high similarity thresholds.
- **Advantages**: Extremely fast (linear complexity).
- **Input**: Pairwise relations in TSV format (`name1  name2  weight`); the weight column is ignored.
- **Output**: `cluster` (default) or `pair` format, controlled by `--format`.
- **CLI options**: `infile`, `--format {cluster|pair}`, `-o/--outfile`.
  - In `pair` format, the representative is the alphabetically first member of each component.

### K-Medoids

- **Principle**: Iterative optimization similar to K-Means, but the center (medoid) must be an actual sample from the dataset. Centers are updated by minimizing the sum of dissimilarities to the nearest center.
- **Command**: `necom clust k-medoids` (alias `km`)
- **Characteristics**: Like K-Means, but centers must be actual samples (medoids).
- **Use cases**: Noise-resistant scenarios, or when only a **distance matrix** (non-Euclidean space) is available.
- **Advantages**: Robust to outliers; interpretable results because centers are real samples.
- **Input**: Pairwise distances `.tsv` (lower is better).
- **Output**: `cluster` (default) or `pair` format, controlled by `--format`.
- **CLI options**: `infile`, `-k/--k <N>` (required), `--format {cluster|pair}`, `--rep {medoid|first}`, `--same <FLOAT>`, `--missing <FLOAT>`, `--runs <N>`, `--max-iter <N>`, `--seed <UINT>`, `-o/--outfile`.
  - `--rep` default: `medoid` (minimum sum of distances to other members); `first` uses the alphabetically first member.
  - `--runs` default: `10` (number of random initializations).
  - `--max-iter` default: `100`.
  - `--same` default: `0.0`; `--missing` default: `1.0`.

### DBSCAN

- **Principle**: Density-based clustering. Starting from any point, if the number of points within its $\epsilon$ neighborhood exceeds `min_points`, it becomes a core point and expands a cluster; regions with insufficient density are treated as noise.
- **Command**: `necom clust dbscan`
- **Characteristics**: Density-based clustering that requires specifying neighborhood radius `eps` and minimum point count `min_points`.
- **Use cases**: **Non-convex** cluster shapes, uneven density distributions, **outlier detection**.
- **Advantages**: Does not require specifying the number of clusters K; identifies noise.
- **Input**: Pairwise distances `.tsv` (lower is better).
- **Output**: `cluster` (one cluster per line, first element is the representative) or `pair` (representative–member pairs).
- **CLI options**: `infile`, `--format {cluster|pair}`, `--rep {medoid|first}`, `--same <FLOAT>`, `--missing <FLOAT>`, `--eps <FLOAT>`, `--min-points <N>`, `-o/--outfile`.
  - `--eps` default: `0.05` (neighborhood radius).
  - `--min-points` default: `4` (minimum points to form a dense region).
  - `--rep` default: `medoid` (smallest sum of distances); `first` uses the alphabetically first member.
  - `--same` default: `0.0`; `--missing` default: `1.0`.
- **Unimplemented options**: Parameter scanning and scoring such as `--scan`, `--opt-eps`, `--min-pct` are not yet implemented; they may be provided later as subcommands of `necom clust dbscan` or standalone scripts.

#### Usage Examples

```bash
# Basic clustering (pairwise distance input)
necom clust dbscan pairs.tsv --eps 0.15 --min-points 3 -o clusters.tsv

# Output pair format for downstream evaluation
necom clust dbscan pairs.tsv --eps 0.15 --min-points 3 --format pair -o pairs.out.tsv
```

### UPGMA

- **Principle**: Unweighted Pair Group Method with Arithmetic Mean. A bottom-up hierarchical clustering that repeatedly merges the two closest clusters; distances between the new cluster and all others are computed as arithmetic averages of all member-to-member distances. Assumes a constant evolutionary rate (molecular clock).
- **Command**: `necom clust upgma`
- **Characteristics**: Hierarchical clustering (average linkage) that outputs a **rooted tree**.
- **Use cases**: Phylogenetic analysis assuming a **molecular clock** (ultrametric).
- **Advantages**: Produces a hierarchical structure with branch heights carrying clear distance meaning.
- **Input**: PHYLIP distance matrix (strict or relaxed).
- **Output**: Newick tree.
- **CLI options**: `infile`, `-o/--outfile`.

### NJ (Neighbor-Joining)

- **Principle**: Neighbor-Joining. Iteratively merges the pair of nodes with the smallest net divergence by minimizing total tree length (based on the Q-matrix-corrected distances). Does not assume a molecular clock and allows different evolutionary rates across branches.
- **Command**: `necom clust nj`
- **Characteristics**: Distance-matrix tree-building algorithm that outputs an **unrooted tree**.
- **Use cases**: General additive distances (no molecular-clock assumption); evolutionary tree construction.
- **Advantages**: Fast; robust to different evolutionary rates.
- **Input**: PHYLIP distance matrix (strict or relaxed).
- **Output**: Newick tree.
- **CLI options**: `infile`, `-o/--outfile`.

### Hierarchical Clustering

- **Principle**: A general bottom-up (agglomerative) clustering framework. Clusters are merged according to different linkage criteria (e.g., Ward minimum variance, Complete maximum distance), building a complete dendrogram hierarchy.
- **Command**: `necom clust hier` (alias `hclust`)
- **Characteristics**: General hierarchical clustering supporting `single`, `complete`, `average`, `weighted`, `centroid`, `median`, `ward`.
- **Implementation status**: Implemented with $O(N^2)$ NN-chain optimization.
- **Value**: Provides a general hierarchical view (not limited to biological evolution); combined with `necom cut` it yields flexible groupings at different granularities.
- **Input**: PHYLIP distance matrix (strict or relaxed).
- **Output**: Newick tree.
- **CLI options**: `infile`, `--method {single|complete|average|weighted|centroid|median|ward}` (default: `ward`), `-o/--outfile`.
- **Details**: [Hierarchical Clustering Details](#hierarchical-clustering-details)

### Tree Cutting

The `necom cut` command splits an existing Newick tree (phylogenetic or hierarchical clustering tree) into flat partitions according to distance, topological, or statistical criteria. It is documented separately in [`docs/cut.md`](cut.md) because it operates on trees rather than running a clustering algorithm.

Trees built by `necom clust hier`, `necom clust upgma`, `necom clust nj`, or external tools can be passed directly to `necom cut`. The resulting partitions can be evaluated with `necom clust eval`.

## Hierarchical Clustering Details

`necom clust hier` (alias `hclust`) provides general hierarchical clustering (dendrogram) generation, supporting `single/complete/average/ward.D2` and other methods, outputting Newick format for downstream `necom cut`.

### Background and Positioning

- **Module**: `clust`, alongside `k-medoids`, `mcl`, etc.
- **Goal**: Statistically meaningful dendrograms (merge heights express the cost of the linkage criterion), without enforcing "evolution/molecular-clock" semantics.
- **Synergy with existing necom capabilities**:
  - Tree building: `clust upgma` (rooted, ultrametric) and `clust nj` (additive, unrooted) already exist.
  - Cutting: tree-cut grouping via `necom cut`.
  - Evaluation: `necom clust eval --matrix` / `--tree` / `--coords` (currently available); `necom nwk eval` not yet implemented.

### Relationship to UPGMA/NJ

- Commonalities: All take a distance matrix as input and output a tree-like structure; all can be combined with `necom cut` to obtain flat groupings.
- Relationship to UPGMA:
  - R `hclust(method="average")` is equivalent to "average linkage"; UPGMA is a specialized version under the "ultrametric (molecular clock)" assumption, producing a rooted, strictly ultrametric tree whose branch lengths have "time/evolution" meaning.
  - Conclusion: The linkage updates are identical, but the semantics differ; UPGMA leans toward phylogenetic scenarios, while `clust hier` leans toward statistical clustering.
- Relationship to NJ:
  - NJ (Neighbor-Joining) minimizes total tree length via the Q matrix, producing an "additive minimum-length tree" that does not belong to the linkage-update paradigm and usually outputs an unrooted tree.
  - For general additive distances, NJ is more robust than UPGMA; if the distances are ultrametric, UPGMA/hclust-average and NJ usually agree topologically (unrooted view).

### Methods and Algorithm Essentials

- `single/complete/average`: Standard linkage updates (Lance–Williams framework); merge height is the distance/cost corresponding to the linkage criterion.
- `ward.D2`:
  - Concept: Minimizes the increase in within-cluster sum of squares (total within-group variance, SSE); commonly used and robust.
  - Update (squared-distance version, where n is cluster size):
    - Let the squared distance between merged cluster `u∪v` and a third cluster `w` be:
    - `d(u∪v,w)^2 = [ (n_u+n_w) d(u,w)^2 + (n_v+n_w) d(v,w)^2 − n_w d(u,v)^2 ] / (n_u+n_v+n_w)`
  - If the input is non-squared distances: square them for the update, and take the square root or use the SSE-increment definition for merge heights when outputting.
  - Distance prerequisite: Theoretically requires Euclidean or near-Euclidean distances; usable on general biological distances, but the statistical interpretation of "variance minimization" becomes weaker.

### Output and Conventions

- Outputs Newick dendrogram:
  - Internal node height is half the merge distance (`height = distance / 2`), and branch length from child to parent is `parent_height - child_height`.
  - Therefore the output is ultrametric-like: all leaves under the same internal node have equal total distance to that node.
  - Branch lengths express merge heights (linkage cost or SSE increment with appropriate unit handling).
  - Strict ultrametricity is not guaranteed (unless the data satisfy the corresponding conditions), but the output satisfies the requirements of `necom cut --height`.
- Numeric format: unified six decimal places with trailing zeros removed; consistent with the convention in `nwk distance`.

### Recommended Hier Workflow

- Generate tree:
  - Near-molecular-clock/ultrametric scenarios: `clust upgma` outputs a rooted ultrametric tree.
  - General additive-distance scenarios: `clust nj`.
  - General hierarchical analysis or when `ward.D2` is needed: `clust hier --method ward.D2`.
- Cut and evaluate:
  - Cut: `necom cut --height H` or TreeCluster-style thresholds/constraints.
  - Internal evaluation (no Ground Truth): `necom clust eval --matrix ...` (Silhouette) (currently available); `necom nwk eval` not yet implemented.
  - External evaluation (with Ground Truth): `necom clust eval` (ARI/AMI/V-Measure).

### CLI Design

#### Command Overview

- Name: `necom clust hier` (visible alias `hclust`)
- Purpose: Generate a hierarchical clustering tree (dendrogram) from a distance matrix, output as Newick for downstream `necom cut`.
- Module: `clust`, alongside `k-medoids` and others.

#### Input

- Matrix file: PHYLIP distance matrix (standard or relaxed format).
- Format conversion: If you have a pair TSV (`name1  name2  distance`), first convert it to PHYLIP with `necom mat to-phylip`; this unified entry reduces ambiguity and keeps the interface consistent with `clust upgma/nj`.
- Distance/similarity conversion: `clust hier` only accepts **distance matrices** (smaller means more similar). If the input is a similarity matrix (e.g., BLAST Identity, Alignment Score), convert it first with `necom mat transform` (e.g., `--op inv-linear --max-val 100` or `--op log`).
- Name source: Parsed automatically from input; no extra label file needed.

#### Main Options

- `--method {single|complete|average|weighted|centroid|median|ward}`: linkage/criterion selection (default `ward`). Naming aligns with SciPy `linkage`.
- `--outfile/-o <path>`: output file path (default `stdout`, i.e., print to screen).

#### Output

- Default output: Newick dendrogram with branch lengths representing merge heights.
- Numeric format: unified six decimal places, trailing zeros removed; consistent with the convention in `nwk distance`.

#### Examples

```bash
# Convert pair TSV to PHYLIP first
necom mat to-phylip pairs.tsv -o matrix.phy

# Ward (PHYLIP input, default Newick output)
necom clust hier matrix.phy --method ward > tree.nwk

# Average/complete/single (PHYLIP input)
necom clust hier matrix.phy --method average > tree.nwk
```

#### Notes

- Distance prerequisite: Ward.D2 theoretically relies on Euclidean or near-Euclidean distances; usable on general biological distances, but the statistical interpretation of "minimum total within-group variance" becomes weaker.
- Semantic differences:
  - Hier merge heights are linkage/criterion costs; ultrametricity is not guaranteed (unless the data satisfy the corresponding conditions).
  - If you need "rooted, ultrametric, evolution-meaningful" branch lengths, use `clust upgma`; for general additive distances, use `clust nj`.
- Stability: Ties are broken by alphabetical order of names to ensure determinism.
- Implementation convention: `ward.D2` internally performs updates with "squared distances" and returns branch lengths in "distance units"; users do not need to provide or distinguish `D` from `D^2`.
- Method characteristics:
  - `centroid/median` may produce non-monotonic merge heights (inversion), which is an algorithmic characteristic; the output is still valid Newick, but the intuitive meaning of heights is slightly weaker than for `average/ward`.
  - Leaf ordering: the `hier` command itself does not reorder leaves; for improved visualization readability, use `necom nwk order --num-descendants` (ladderize).

### Mapping to and Differences from SciPy

- Method mapping: Aligned with SciPy `linkage` `method` set; `ward` is equivalent to `ward.D2` (internally updates with squared distances); `average` is equivalent to UPGMA, `weighted` to WPGMA, and `centroid/median` to UPGMC/WPGMC.
- Input differences: SciPy accepts a "condensed distance vector" or an observation matrix; necom uniformly uses PHYLIP distance matrices. To convert from pair TSV, use `necom mat to-phylip`.
- Output differences: SciPy returns an `(n-1)×4` linkage matrix Z; necom outputs a Newick tree directly for `necom cut / to-dot / to-forest`. Average users do not need to care about Z; if SciPy interoperability is required, continue using Z with `fcluster/cophenet` on the Python side.
- Leaf ordering: necom recommends `necom nwk order --num-descendants` (ladderize) for extremely high performance and usually sufficient visualization quality.
- Flat clustering: SciPy's `fcluster` supports `criterion='distance'|'maxclust'|...`; in necom these correspond to `necom cut --height H` and `necom cut --k K`, respectively. Other criteria such as `monocrit/inconsistent` are not introduced for now.
- Evaluation metrics: SciPy has `cophenet` (cophenetic correlation coefficient); necom plans to add the cophenetic correlation coefficient to `necom nwk eval` as a supplementary tree-quality metric (not yet implemented).

#### User Tips

- Beginner path (recommended): `mat to-phylip → clust hier --method ward → necom cut --height → clust eval → nwk visualization`
- Interoperability and audit: If you need to verify the merging process step by step or perform further flat cutting/statistics in Python, use SciPy's linkage matrix and tools; on the necom side, keep Newick as the primary format to reduce cognitive load.

#### Example Mappings

- SciPy linkage (Ward):
  - Python: `Z = linkage(y, method='ward', optimal_ordering=True)`
  - necom: `necom mat to-phylip pairs.tsv -o matrix.phy` → `necom clust hier matrix.phy --method ward > tree.nwk` → `necom nwk order tree.nwk --num-descendants > ordered.nwk`
- SciPy fcluster (cut by distance):
  - Python: `labels = fcluster(Z, t=0.05, criterion='distance')`
  - necom: `necom cut tree.nwk --height 0.05 > clusters.tsv`
- SciPy fcluster (cut by cluster count):
  - Python: `labels = fcluster(Z, t=20, criterion='maxclust')`
  - necom: `necom cut tree.nwk --k 20 > clusters.tsv`
- SciPy cophenet:
  - Python: `c, dists = cophenet(Z, Y)`

#### scikit-learn Mapping

- AgglomerativeClustering (Ward):
  - Python: `model = AgglomerativeClustering(linkage='ward').fit(X)`
  - necom: `necom clust hier matrix.phy --method ward > tree.nwk` (distance matrix must be computed first)
- AgglomerativeClustering (Average/Complete/Single):
  - Python: `model = AgglomerativeClustering(linkage='average').fit(X)`
  - necom: `necom clust hier matrix.phy --method average > tree.nwk`
- Differences:
  - scikit-learn focuses on directly outputting cluster labels (`labels_`); `necom` focuses on generating tree structure (Newick).
  - To obtain labels in `necom`, use it together with `necom cut`.

#### Toolchain Collaboration

- Tree building: `necom clust hier` → generate dendrogram
- Cutting: `necom cut --height H` → export groups
- Evaluation:
  - No Ground Truth: `necom clust eval --matrix` / `--tree` / `--coords` (currently available); `necom nwk eval` not yet implemented
  - With Ground Truth: `necom clust eval` (ARI/AMI/V-Measure)
- Visualization: `necom nwk to-dot/to-forest` → graphic/LaTeX display

## Evaluation and Analysis

These commands do not produce clusters; they evaluate cluster or tree quality.

- **Tree-based evaluation**
  - **Command**: `necom nwk eval` (not yet implemented)
  - **Positioning**: Multi-dimensional evaluation of tree structure.
  - **Capabilities**: Geometric compactness (Silhouette), taxonomic purity (Purity), evolutionary consistency (Discordance).
  - **Alternative**: Currently use `necom clust eval --matrix` / `--tree` / `--coords` for distance/tree/coordinate-based evaluation.

- **Partition-based evaluation**
  - **Command**: `necom clust eval`
  - **Positioning**: General clustering quality evaluation (supports with/without Ground Truth).
  - **Capabilities**: ARI, AMI, V-Measure (external); Silhouette, Davies-Bouldin (internal).

`necom` separates cluster generation from evaluation: generate candidates with `necom cut --scan`, then evaluate them in batch with `clust eval`, and finally select the best parameters manually.

### External Validity Metrics

*Compare a predicted partition to a reference partition.*

| Metric | Range | Notes |
| :--- | :--- | :--- |
| ARI | [-1, 1] | Adjusted Rand Index; corrected for chance; 1 = perfect agreement. |
| AMI | [0, 1] | Adjusted Mutual Information; robust with many clusters. |
| NMI | [0, 1] | Normalized Mutual Information; not corrected for chance. |
| V-Measure | [0, 1] | Harmonic mean of Homogeneity and Completeness. |
| FMI | [0, 1] | Fowlkes-Mallows Index; geometric mean of Precision and Recall. |
| RI | [0, 1] | Rand Index; not corrected for chance. |
| Jaccard | [0, 1] | Pair-wise Jaccard similarity of same-cluster pairs. |
| Precision / Recall | [0, 1] | Pair-wise precision and recall. |

### Internal Validity Metrics

*Evaluate a partition using the data geometry only.*

| Type | Metrics | Required input |
| :--- | :--- | :--- |
| Distance-based | Silhouette, Dunn, C-Index, Gamma, Tau | `--matrix` or `--tree` |
| Coordinate-based | Davies-Bouldin, Calinski-Harabasz, PBM, Ball-Hall, Xie-Beni, Wemmert-Gancarski | `--coords` |

Silhouette is the most commonly used distance-based metric: values near 1 indicate well-clustered samples, 0 indicate boundary samples, and negative values suggest possible mis-clustering.

### Typical Workflows

```bash
# External evaluation (with ground truth)
necom clust eval result.tsv --other truth.tsv -o eval.tsv

# Internal evaluation with a distance matrix
necom clust eval result.tsv --matrix dist.phy

# Batch evaluation of scan results
necom cut tree.nwk --height 1.0 --scan 0,1.0,0.05 | \
    necom clust eval - --input-format long --matrix matrix.phy > evaluation.tsv
```

## Planned

GMM, HDBSCAN, Louvain/Leiden and other algorithms are on the roadmap.

## Not Recommended / No Plans

These algorithms are classic but have limitations in large-scale biological data scenarios, so they are not being introduced as core features.

- **K-Means**
  - **Reason**: Fast, but assumes clusters are spherical with equal variance, and centroids are usually not real samples, lacking biological interpretability (e.g., they cannot directly serve as representative sequences).
  - **Alternative**: `K-Medoids` (already implemented), where medoids must be real samples and arbitrary distance matrices are supported, making it more suitable for biological sequence analysis.

- **Bisecting K-Means**
  - **Principle**: Top-down divisive clustering. Initially all points form one cluster; the cluster with the largest SSE is selected and split by K-Means until K clusters are reached.
  - **Reason**: Although it produces a tree structure (binary tree), it inherits K-Means limitations (requires Euclidean distances; centroids are not real samples). Biological tree building usually prefers bottom-up agglomerative methods such as UPGMA/NJ.

- **Affinity Propagation (AP)**
  - **Principle**: Message-passing mechanism in which all points compete to become exemplars. Does not require specifying the cluster count, but has high computational complexity.
  - **Reason**: Time and space complexity are high ($O(N^2)$), making it difficult to handle large-scale biological sequence data (e.g., >10k sequences).
  - **Alternative**: For small datasets seeking representatives, use `K-Medoids`; for automatic cluster count, use `DBSCAN` or `MCL`.

- **Spectral Clustering**
  - **Principle**: Uses eigenvectors of the Laplacian matrix for dimensionality reduction, then performs K-Means clustering in the low-dimensional space. Essentially seeks the graph's minimum normalized cut (Normalized Cut).
  - **Reason**: Constructing the Laplacian matrix and eigendecomposition is computationally expensive ($O(N^3)$).
  - **Alternative**: `MCL` usually provides similar or better results for biological network clustering and scales better.

- **Mean Shift**
  - **Principle**: Density-based hill-climbing algorithm. Points are repeatedly shifted toward the density center (mean shift) of their neighborhood until they converge to local density peaks (modes).
  - **Reason**: High computational complexity and the bandwidth parameter is hard to choose adaptively.
  - **Alternative**: `DBSCAN` or `GMM` usually covers its density-estimation needs.

- **OPTICS**
  - **Principle**: Produces a reachability plot by ordering data points according to reachability distances, capturing all possible density levels in a single run. Solves DBSCAN's sensitivity to a global `eps`.
  - **Reason**: Its core idea (hierarchical density clustering) has been better inherited and automated by **HDBSCAN**; OPTICS results (reachability plots) require complex post-processing to obtain clear clusters.
  - **Alternative**: Use the more modern, lower-parameter, and more automated `HDBSCAN`.

- **Biclustering**
  - **Reason**: Simultaneously clusters rows and columns (e.g., Spectral Co-Clustering), mainly used for specific matrix-subblock mining scenarios such as gene expression profiling. This differs substantially from necom's current focus on "sample grouping".
  - **Alternative**: If features (columns) need to be clustered, transpose the matrix and use standard clustering; if co-expression modules are needed, use dedicated expression-profiling tools (e.g., WGCNA).

- **BIRCH**
  - **Principle**: Incremental clustering based on a Clustering Feature tree (CF Tree). Builds a highly compressed tree in a single scan; tree nodes store cluster statistics (sum, square sum), making it ideal for very large datasets.
  - **Reason**: Strongly relies on Euclidean-space statistical properties (computing centroids and radii), not suitable for complex distance measures of biological sequences (e.g., edit distance); also restricts cluster shapes.
  - **Alternative**: For large-scale vectors, MiniBatch K-Means is more general; for large-scale sequences, use `MCL` (graph clustering) or `CD-HIT/MMseqs2` (greedy clustering).

## Detailed Algorithm Descriptions

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
- **Silhouette / Calinski-Harabasz** [Partially supported]: Geometry-based evaluation metrics suitable for K-means or general distance clustering (`clust eval` already supports distance-matrix Silhouette; tree-based Silhouette is planned for `necom nwk eval` [planned]).

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

# 2. Extract representative list
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

Combine `necom cut` scanning with `clust eval` batch evaluation to find the best cutting threshold.

```bash
# 1. Generate hierarchical clustering tree
necom clust hier matrix.phy --method ward > tree.nwk

# 2. Scan thresholds and evaluate internal metrics (Silhouette)
# necom cut outputs a long table in scan mode, which can be piped directly to clust eval
necom cut tree.nwk --height 1.0 --scan 0,1.0,0.05 | \
    necom clust eval - --input-format long --matrix matrix.phy > evaluation.tsv

# 3. Analyze evaluation.tsv to choose the best threshold (e.g., maximum Silhouette)
# Assume the best threshold is 0.45
necom cut tree.nwk --height 0.45 > final_clusters.tsv
```

## Input/Output Format Conventions

The `necom clust` family of commands involves multiple data formats; the following standard formats are used for interoperability.

### 1. Partition Files

Used to represent clustering results (sample-to-cluster mapping). Three formats are supported via the `--format` option.

#### Pair Format (default, `--format pair`)
The most general long-table format; each line indicates which cluster a sample belongs to.
- **Structure**: `ClusterID <tab> Item`
- **Representative selection**: For `dbscan`/`mcl`/`k-medoids`, controlled by `--rep {medoid|first}`; default `medoid` (extreme of sum of distances/similarities), `first` is the alphabetically first member in the cluster. This option also affects the first column of `cluster` format. `cc` does not read weights and always uses `first`.
- **Characteristics**: Easy to parse; supports streaming.
- **Example**:
  ```text
  # Numeric ID
  1	GeneA
  1	GeneB
  2	GeneC

  # Representative as ID
  GeneA	GeneA
  GeneA	GeneB
  GeneC	GeneC
  ```

#### Cluster Format (`--format cluster`)
Wide-table format; each line represents a cluster containing all its members.
- **Structure**: `Item1 <space/tab> Item2 ...`
- **Characteristics**: Human-readable; suitable for inspecting results. The line number (1-based) is the ClusterID.
- **Example**:
  ```text
  GeneA GeneB
  GeneC
  ```

#### Long Format (batch, `--format long`)
A dedicated format for batch evaluation. `--input-format long` is only accepted by `necom clust eval`; `necom cut`'s `--format` only supports `cluster`/`pair`, but in `--scan` mode it automatically outputs the long format.
- **Structure**: `Group <tab> ClusterID <tab> Item`
- **Group column**: Identifies different parameter combinations or cutting methods. The format is usually `Method=Value` (e.g., `height=0.5`).
  - `necom clust eval` preserves this column as the identifier in evaluation results.
- **Example**:
  ```text
  height=0.1	1	GeneA
  height=0.1	2	GeneC
  height=0.2	1	GeneA
  height=0.2	1	GeneC
  ```

### 2. Distance Matrix

Used by `clust hier`, `nj`, `upgma`, and `eval --matrix`.

#### PHYLIP Format (relaxed)
- **Structure**:
  - First line: sample count $N$.
  - Next $N$ lines: `Name <space> Dist1 <space> Dist2 ...`
- **Characteristics**: Standard bioinformatics format. `necom` supports a "relaxed" format with arbitrary whitespace between names and data.
- **Example**:
  ```text
  3
  A  0.0 0.1 0.5
  B  0.1 0.0 0.5
  C  0.5 0.5 0.0
  ```

### 3. Coordinates / Feature Vectors

Used by `clust eval --coords` (Davies-Bouldin Index) or future `kmeans/gmm`.

#### FeatureVector Format
- **Structure**: `Name <tab> Val1,Val2,Val3...`
- **Delimiters**: **Tab** between name and vector; **commas** between numeric values.
- **Example**:
  ```text
  GeneA	1.2,0.5,3.3
  GeneB	1.1,0.6,3.1
  ```
- **Compatibility**: A general feature-vector/coordinate representation format.
