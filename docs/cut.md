# necom cut

`necom cut` cuts a Newick tree (phylogenetic or hierarchical clustering tree) into flat clustering partitions.

Unlike `necom clust` (which builds clusters from data), `cut` focuses on "deriving groups from an existing tree structure." It supports multiple biological and statistical cutting rules and provides stable, reusable tabular output.

This document describes the command's algorithm patterns, parameter-selection guidelines, and input/output conventions.

## Use Cases and Design Rationale

In practice, we often already have a tree (phylogenetic or hierarchical clustering tree) and want to cut its leaves into different groups (partitions) at some threshold. Cutting rules can vary: by height, by number of clusters, by maximum within-cluster distance (diameter), requiring monophyly (clade), etc.

`necom cut` aims to provide a comprehensive, efficient, and standardized cutting toolkit:

- **Algorithm core**:
  Implements basic cuts by cluster count (`--k`) and height (`--height`), with logic consistent with `SciPy.cluster.hierarchy` and R `cutree`; it also fully ports TreeCluster's biological-constraint algorithms (e.g., `--max-clade`, `--med-clade`), optimized for phylogenetic trees.

- **Performance and usability improvements**:
  - **High performance**: Rust-based implementation with no external dependencies, handling large trees more efficiently.
  - **Standardized**: Unifies terminology differences across source algorithms (e.g., consistently using `--height`, `--single-linkage`), reducing cognitive load.
  - **Composable**: As part of the `necom` toolchain, it can directly cooperate with `eval partition` for clustering evaluation.

## Supported Modes and Algorithms

`necom cut` provides a rich set of cutting algorithms, ranging from simple threshold cuts to complex biologically constrained clustering. Detailed definitions and complexity analyses are given below.

### 1. Cut by Cluster Count (`--k <K>`)

- **Definition**: Split the tree into $K$ clusters produced by $K-1$ cuts. Cutting order is based on node height (distance to the farthest leaf), prioritizing nodes with the largest height.
- **Complexity**: $O(N \log N)$, where $N$ is the number of leaves. Requires sorting all internal nodes by height.
- **Use case**: Exploratory analysis when you only want a fixed number of groups and do not care about the exact distance threshold.

### 2. Cut by Height (`--height <H>`)

- **Definition**: Cut all edges whose node height (distance to the farthest leaf) is greater than $H$.
  - For any resulting cluster $C$, all nodes $u$ in it satisfy $height(u) \le H$.
  - Equivalent to SciPy's `fcluster(criterion='distance')` or R's `cutree(h=H)`.
- **Complexity**: $O(N)$. Only one post-order traversal is needed.
- **Use case**: Suitable for ultrametric trees, where height strictly represents time or genetic distance.

### 3. Cut by Root Distance (`--root-dist <D>`)

- **Definition**: Cut all edges whose path length from the root exceeds $D$.
  - For the root node $r_C$ of any resulting cluster $C$, $dist(root, r_C) \le D$.
  - Once a path's cumulative length exceeds $D$, that path is cut and no longer extends downward.
- **Complexity**: $O(N)$. Only one pre-order traversal is needed.
- **Use case**: Phylogenetic analysis defining clades that diverged a certain time after the common ancestor (root).

### 4. Cut by Maximum Within-Cluster Diameter (`--max-clade <T>`)

- **Definition**: Partition leaves into non-overlapping clusters $\{C_1, C_2, ..., C_m\}$ such that for each cluster $C_i$:
  1. **Monophyly**: Leaves in $C_i$ must form a clade in the original tree $T$.
  2. **Diameter constraint**: $\max_{u, v \in C_i} dist(u, v) \le T$.
  3. **Support constraint** (if `--support` is specified): The path between any two points in $C_i$ must not contain edges with support below the threshold.
- **Algorithm**: TreeCluster "Max Clade" algorithm. Uses efficient bottom-up diameter computation and top-down greedy selection.
- **Complexity**: $O(N)$. Avoids the $O(N^2)$ all-pairs distance computation.
- **Use case**: Virus subtyping, OTU clustering, and other scenarios requiring strict control of within-cluster divergence.

### 5. Cut by Average Within-Cluster Distance (`--avg-clade <T>`)

- **Definition**: Similar to `--max-clade`, but the constraint is:
  1. **Monophyly**.
  2. **Average distance constraint**: $\frac{1}{|C_i|(|C_i|-1)} \sum_{u, v \in C_i, u \neq v} dist(u, v) \le T$.
  3. **Support constraint**.
- **Algorithm**: TreeCluster "Avg Clade" algorithm. Maintains within-subtree distance sums and node counts bottom-up.
- **Complexity**: $O(N)$.
- **Use case**: Compared to maximum distance, average distance is more robust to individual outliers.

### 6. Cut by Median Within-Cluster Distance (`--med-clade <T>`)

- **Definition**: Similar to `--max-clade`, but the constraint is:
  1. **Monophyly**.
  2. **Median distance constraint**: $median(\{dist(u, v) \mid u, v \in C_i, u \neq v\}) \le T$.
  3. **Support constraint**.
- **Algorithm**: TreeCluster "Med Clade" algorithm. Computes medians by merging sorted lists bottom-up.
- **Complexity**: $O(N^2 \log N)$ in the worst case. Significantly more expensive than the previous two methods.
- **Note**: Not recommended for very large trees (e.g., >10k leaves) unless median robustness is truly required.

### 7. Cut by Total Within-Cluster Branch Length (`--sum-branch <T>`)

- **Definition**: Similar to `--max-clade`, but the constraint is:
  1. **Monophyly**.
  2. **Total branch length constraint**: The total branch length of the minimal subtree spanning cluster $C_i$ (Phylogenetic Diversity, PD) $\le T$.
  3. **Support constraint**.
- **Biological meaning**: Total branch length corresponds to **Phylogenetic Diversity (PD)**, representing the total evolutionary history contained in the cluster.
- **Algorithm**: TreeCluster "Sum Branch Clade" algorithm.
- **Complexity**: $O(N)$.
- **Notes**:
    - PD is an **extensive quantity** (monotonically increasing with sample size), unlike diameter or average distance, which are "intensive quantities."
    - Therefore, as a cutting threshold it tends to chop tight large clusters (because accumulated branch length easily exceeds the limit) while retaining loose small clusters.
    - Unless there is a specific biological reason (e.g., "limit the maximum evolutionary potential of each OTU"), it is generally not recommended as the primary cutting criterion.

### 8. Cut by Leaf Distance (`--leaf-dist-max/min/avg <T>`)

- **Definition**: Cutting based on the distance from the cluster root to leaves.
  - **Max Leaf Dist** (`--leaf-dist-max <T>`): Cut the tree so that the maximum distance from the cluster root to any leaf is $\le T$.
    - Equivalent to `root_dist(max_depth - T)`.
    - Similar to `--height`, but suitable for non-ultrametric trees, aligned by the farthest leaf.
  - **Min Leaf Dist** (`--leaf-dist-min <T>`): Cut the tree so that the minimum distance from the cluster root to any leaf is $\le T$.
    - Equivalent to `root_dist(min_depth - T)`.
  - **Avg Leaf Dist** (`--leaf-dist-avg <T>`): Cut the tree so that the average distance from the cluster root to all leaves is $\le T$.
    - Equivalent to `root_dist(avg_depth - T)`.
- **Complexity**: $O(N)$. Requires a prior traversal to compute depth statistics.
- **Use case**: Non-ultrametric trees (e.g., virus trees), where "time" is not uniform and one needs to look back from sampling times (leaves).

### 9. Cut by Maximum Edge Length (`--max-edge <T>`)

- **Definition**: Cut all edges whose length is greater than $T$.
  - For any resulting cluster $C$, all edges $e$ in it satisfy $length(e) \le T$.
  - This method is also known in graph theory as **Single Linkage Clustering**: as long as two points are connected by a path of "short edges" ($\le T$), they belong to the same cluster.
  - Alias: `--single-linkage <T>`.
- **Complexity**: $O(N)$. Only one traversal is needed.
- **Use cases**:
  - Remove the influence of long branch attraction.
  - Quickly identify tightly connected groups while ignoring sparse connections.

### 10. Cut by Inconsistency Coefficient (`--inconsistent <T>`)

- **Definition**: Cut based on the "inconsistency" of a node relative to its subtrees.
  - For each non-leaf node $i$, compute its inconsistency coefficient $I_i$.
  - $I_i = \frac{height(i) - \text{mean}(H)}{\text{std}(H)}$, where $H$ is the set of all merge heights within $d$ levels (`--deep`) below node $i$.
  - **SciPy reference**: `scipy.cluster.hierarchy.inconsistent`.
  - Traverse from the root downward for each node $i$:
    - Compute the maximum inconsistency coefficient $M_i$ in the subtree rooted at $i$.
    - If $M_i \le T$, the subtree is sufficiently "consistent"; treat $i$ and its leaves as one cluster and stop splitting.
    - If $M_i > T$, the subtree contains significantly inconsistent merge points; continue checking $i$'s children.
  - Leaf nodes naturally form a cluster.
- **Complexity**: $O(N)$.
- **Use case**: When overall evolutionary rates are uneven, find "natural" cluster boundaries.

### 11. Dynamic Tree Cut (`--dynamic-tree`) [Implemented]

- **Definition**: Based on the `cutreeDynamicTree` algorithm in the R package `dynamicTreeCut` (`dynamicTreeCut/R/cutreeDynamic.R`).
- **Principle**: Top-down recursive algorithm.
  1.  First perform an initial cut based on global height.
  2.  For each preliminary cluster, analyze its internal structure (height distribution).
  3.  If a cluster contains significant substructure (i.e., a split point satisfying conditions for "height difference" and "subcluster size"), recursively split it further.
- **Options**:
  - `--dynamic-tree <N>`: Enable dynamic tree cut; N is the minimum cluster size (required, no default).
  - `--deep-split`: Enable more aggressive splitting (corresponds to `deepSplit=TRUE` in R).
  - `--max-tree-height <H>` (optional): Maximum merge height (default 99% tree height).
- **Input**: Only the tree structure (dendrogram) is needed; no distance matrix is required.
- **Use cases**:
  - Only tree structure is available; fast, automated cutting is desired.
  - Suitable for nested structures with small clusters inside large clusters.

### 12. Hybrid Dynamic Cut (`--dynamic-hybrid`) [Implemented]

- **Definition**: Based on the `cutreeHybrid` algorithm in the R package `dynamicTreeCut`.
- **Principle**: Two-phase bottom-up algorithm.
  1.  **Core Detection**:
      - Implements the bottom-up algorithm of R `cutreeHybrid`, identifying "core clusters" that satisfy tightness (Core Scatter) and separation (Gap) requirements.
  2.  **PAM-like Reassignment**: Uses the original distance matrix to adsorb objects left unassigned in the first phase (outliers/singletons) to the nearest core cluster (medoid-based assignment).
      - Default behavior is consistent with R (`pamStage=TRUE`, `pamRespectsDendro=TRUE`, `respectSmallClusters=TRUE`).
- **Input**: Requires both tree structure and the original distance matrix (`--matrix`).
- **Options**:
  - `--dynamic-hybrid <N>`: Enable hybrid cut; N is the minimum cluster size.
  - `--matrix <FILE>`: Distance matrix file (PHYLIP format).
  - `--max-pam-dist <D>`: Maximum distance threshold for PAM assignment (default equals the cut height `cutHeight`). If an unassigned point's distance to the nearest medoid exceeds this value, it remains unassigned.
  - `--no-pam-dendro`: Disable the dendrogram constraint in the PAM stage (allows assigning objects across high branches). By default the PAM stage respects tree structure (`pamRespectsDendro=TRUE`).
- **Use cases**:
  - High accuracy at cluster boundaries is required.
  - The distance matrix is needed to correct small errors or uncertainties in the tree structure.
  - Can effectively identify and handle outliers.

### 13. Support Filtering (`--support <S>`)

- **Definition**: A **preprocessing step** for all the above methods.
  - Traverse all edges in the tree; if an edge's support value $< S$, its length is treated as $+\infty$ (infinite).
  - **Default behavior**: For nodes without explicit support values (e.g., internal nodes produced by parsing multifurcating trees), `necom` defaults their support to 100 (fully trusted).
- **Effect**: Any clustering attempt that crosses a low-support edge will fail because the distance/height exceeds the limit, thereby forcing a cut at low-support positions.

## Input and Output

### Input

- **Input tree**: Newick format (single tree).
- **Branch lengths**: Used for distance/height-related methods (e.g., root distance, max pairwise distance).
- **Branch support (optional)**: If nodes/edges carry support values (e.g., bootstrap), they can be used as a "non-crossable" constraint.

### Output

Output follows the same conventions as `necom clust dbscan` for interoperability with existing tools:

- `cluster` format: Each line contains all points of one cluster; the first point is the representative.
- `pair` format: Each line contains a pair (representative, cluster member).
  - Representative: the representative point of the cluster.
  - Member: a cluster member.
  - Singleton: the representative is itself.

**Representative selection (`--rep`)**:
Applies to both `cluster` and `pair` formats:
- `root` (default): the member closest to the root (alphabetical order as tie-break).
- `medoid`: Medoid, i.e., the member with the smallest sum of distances to other members.
- `first`: the alphabetically first member.

### Common Options

- `--format {cluster|pair}`: Output format. Default: `cluster`.
- `--rep {root|medoid|first}`: Representative selection. Default: `root`.
- `--deep <N>`: Depth used by the inconsistency coefficient method (`--inconsistent`). Default: `2`.
- `--support <S>`: Treat edges with support `< S` as infinite length, forcing a cut. Default behavior treats unlabeled internal nodes as support `100.0`.
- `--scan <start>,<end>,<step>`: Parameter sweep over the main threshold.
- `--stats-out <FILE>`: In scan mode, write the summary statistics table to this file.

## Workflow and Toolchain Collaboration

To keep commands focused and orthogonal, we recommend the following "generate-evaluate" separated workflow:

### 1. Generate

Use `necom cut`:
- It only "cuts"; it does not "evaluate."
- Supports multiple strategies (k, height, max_clade, etc.) and parameter scanning.
- Outputs standard TSV format.

### 2. Evaluate

Evaluating clustering quality usually requires a reference standard (Ground Truth) or comparison with other results. This logic is placed in separate `necom eval` commands; tree-topology comparison is also available via `necom eval compare`:

- **General metrics (`necom eval partition`, `necom mat compare`)**:
  - Input: two clustering result TSVs (or one result + one reference); or two distance matrices for Cophenetic correlation.
  - Output: ARI (Adjusted Rand Index), AMI (Adjusted Mutual Information), V-Measure, etc.; or matrix similarity scores from `mat compare`.
  - Use case: When the true classification of samples is known, or when you want to compare the difference between two cutting parameters.

- **Tree-related metrics (`necom eval tree` [planned])**:
  - Input: tree file + clustering result.
  - Output: Parsimony score, Silhouette score (based on tree distance matrix), etc.
  - Use case: No true classification is available; assess compactness or separability of clusters on the tree structure.

### Recommended Workflow Examples

#### 1. Classic Phylogenetic Analysis
```bash
# 1. Scan different parameters to generate multiple clustering results
# necom cut input.nwk --max-clade 0.10 --scan 0.01,0.10,0.01 > partitions.tsv

# 2. Select the best threshold and generate final clusters
necom cut input.nwk --max-clade 0.05 > final_cluster.tsv

# 3. Extract the subtree corresponding to the first cluster in final_cluster.tsv
head -1 final_cluster.tsv | tr '\t' '\n' > cluster1.names
necom nwk subtree input.nwk -l cluster1.names > cluster1.nwk
```

#### 2. Hierarchical Clustering (hclust) Integration

Starting from a distance matrix, generate a tree with `necom clust hier`, cut it, and evaluate. A complete worked example (hier → cut --scan → eval partition) is in [`docs/clust.md`](clust.md#scenario-b-hierarchical-clustering-parameter-scanning-and-evaluation-workflow).

#### 3. SciPy-style Analysis (Inconsistency Coefficient)
For trees with uneven evolutionary rates, the inconsistency coefficient can find more natural cluster boundaries.

```bash
# Cut using inconsistency coefficient (default depth=2)
necom cut tree.nwk --inconsistent 1.5 > clusters.tsv
```

## Choosing Threshold / Cluster Count: Scanning and Criteria

In `cut` scenarios, users commonly make two types of choices:

- Directly specify the cluster count `K` (analogous to R `cutree(k=...)`).
- Specify a threshold `t` (distance/height/diameter etc.) and let the threshold determine the cluster count.

When unsure about `K` or `t`, use `--scan` for parameter scanning.

### Scanning

`necom` provides explicit scanning capability: applicable to all numeric-parameter methods (e.g., `--k`, `--height`, `--max-clade`, `--inconsistent`).

**Usage**:
`necom cut ... --scan <start>,<end>,<step>`
(Note: scanning only targets the **main threshold parameter** of the method. For `--inconsistent`, it scans the coefficient threshold `T`, while depth `--deep` remains fixed at the user-specified or default value.)

**Output summary table**:
| Group | Clusters | Singletons | Non-Singletons | Max Cluster Size |
| :--- | :--- | :--- | :--- | :--- |
| height=0.01 | 500 | 480 | 20 | 5 |
| height=0.02 | 300 | 200 | 100 | 15 |
| ... | ... | ... | ... | ... |

- **Non-Singletons**: The metric that TreeCluster `argmax_clusters` tries to maximize.
- **Max Cluster Size**: Helps judge whether a "super-cluster" exists (under-clustering).

### Output Format in Scan Mode

When `--scan` is enabled, the `--format` option is ignored. Output behavior is as follows:

1.  **Standard output (`stdout` or `-o`)**: Always outputs a detailed partition table (Long format / Tidy Data).
    - Column definitions: `Group`, `ClusterID`, `SampleID`.
    - The `Group` column is formatted as `Method=Value` (e.g., `height=0.5`, `max-clade=0.02`), making it easy to distinguish different cutting parameters.
    - This format can be directly used as input for `necom eval partition --input-format long` for batch evaluation.
2.  **Statistics output (`--stats-out`)**: If specified, writes the summary statistics table (threshold, cluster count, singleton count, non-singleton count, max cluster size) to that file.

Example:
```bash
# 1. Output only the detailed partition table (for downstream analysis or evaluation)
necom cut tree.nwk --max-clade 0.5 --scan 0,0.5,0.01 > partitions.tsv

# 2. Save statistics at the same time (for quick inspection)
necom cut tree.nwk --max-clade 0.5 --scan 0,0.5,0.01 -o partitions.tsv --stats-out stats.tsv
```

### Integration with `necom eval partition`

`necom cut` and `necom eval partition` work together through the Long format, supporting two evaluation modes:

* **Batch internal evaluation**: pipe scan output to `necom eval partition --input-format long` with `--matrix`, `--tree`, or `--coords` to score every threshold without Ground Truth.
* **Targeted external evaluation**: first use `--scan` to locate promising threshold ranges, then generate partitions for selected thresholds and compare them to Ground Truth with `necom eval partition --other`.

Concrete command examples are kept in `docs/clust.md` to avoid duplication.

### Threshold-Selection Strategy Reference

When unsure about the best threshold, use `--scan` to generate data and refer to the following common strategies for decision-making:

#### Strategy 1: Maximize Non-Singleton Clusters

- **Principle**: Find a threshold that maximizes the number of "non-singleton clusters" (clusters with >1 member).
- **Applicability**: When you expect as many meaningful (>1 member) clusters as possible while avoiding over-chopping (many singletons) or under-cutting (huge clusters).
- **Operation**: Observe the `Non-Singletons` column in the scan result table and choose the threshold corresponding to its maximum.

#### Strategy 2: Elbow Rule
This is a general strategy in data analysis.

- **Principle**: Look at the curve of threshold versus cluster count (or singleton count) and find the "elbow" point.
  - **Steep decline phase**: As the threshold relaxes, cluster count drops rapidly (many tiny clusters merge).
  - **Flat plateau phase**: Cluster count changes stabilize.
  - **Elbow point**: The point where the curve changes from "steep" to "flat," usually corresponding to the data's inherent natural structure.
- **Operation**:
  1. Run scan: `necom cut ... --scan ... > scan.tsv`
  2. Observe the rate of change: if cluster count changes sharply when the threshold increases from $T_1$ to $T_2$ but flattens from $T_2$ to $T_3$, then $T_2$ may be the best cut point.
  3. Visualization: Import `scan.tsv` into plotting tools to assist judgment.

#### Strategy 3: Evaluation-Metric Based
This is the most rigorous strategy, using `necom eval partition` to compute clustering quality metrics.

- **Principle**: Directly compute internal validity (e.g., Silhouette) or external consistency (e.g., ARI, if Ground Truth is available) of the partition.
- **Operation**: Use together with `necom eval partition`.
  ```bash
  # Generate detailed list of all candidate partitions
  necom cut ... --scan ... > partitions.tsv
  # Batch evaluation
  necom eval partition partitions.tsv --input-format long --matrix dist.phy
  ```

## Existing Tool References

The design of `necom cut` draws on best practices from multiple fields:

- **SciPy (`scipy.cluster.hierarchy`)**:
  - Provides the `fcluster` function, supporting cuts by height (`distance`), cluster count (`maxclust`), and inconsistency coefficient (`inconsistent`).
  - `necom` reuses its definitions of `height` and `inconsistent`.

- **R (`dynamicTreeCut`)**:
  - Provides `cutreeDynamic` (Tree) and `cutreeHybrid` (Hybrid) methods.
  - Introduces the ideas of "adaptive recursive cutting" and "core detection + reassignment."
  - `necom` fully implements its Dynamic Tree and Hybrid algorithms, providing a high-performance Rust version.

- **TreeCluster**:
  - Designed specifically for phylogenetic trees, introducing `Max Clade` (diameter), `Avg Clade`, etc.
  - Solves cutting problems for non-ultrametric trees.
  - `necom` fully implements its core algorithm set.

- **R (`cutree`)**:
  - Provides the most basic `h` (height) and `k` (cluster count) cuts.
