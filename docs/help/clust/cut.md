Cut a tree into clusters based on various criteria.

Input:

* A Newick tree file.

Criteria:

* `--k <K>`: cut into K clusters (top-down split by height).
* `--height <H>`: cut at specific height (max distance to leaves).
* `--root-dist <D>`: cut at specific distance from root.
* `--max-clade <T>`: TreeCluster style (max pairwise distance in clade <= T).
* `--avg-clade <T>`: TreeCluster style (avg pairwise distance in clade <= T).
* `--med-clade <T>`: TreeCluster style (median pairwise distance in clade <= T).
* `--sum-branch <T>`: TreeCluster style (sum of branch lengths in clade <= T).
* `--leaf-dist-max <T>`: TreeCluster style (max distance from cluster root to any leaf <= T).
* `--leaf-dist-min <T>`: TreeCluster style (min distance from cluster root to any leaf <= T).
* `--leaf-dist-avg <T>`: TreeCluster style (avg distance from cluster root to leaves <= T).
* `--max-edge <T>` / `--single-linkage <T>`: cut branches longer than threshold.
* `--inconsistent <T>`: SciPy style (inconsistent coefficient <= T).
* `--dynamic-tree <N>`: Dynamic Tree Cut (top-down adaptive, N=min cluster size).
* `--dynamic-hybrid <N>`: Hybrid Cut (Dynamic Tree + PAM, N=min cluster size).

Output:

* `--format cluster` (default): each line contains points of one cluster. The first point is the representative.
* `--format pair`: each line contains a `(representative, member)` pair.

Notes:

* The representative point is determined by `--rep` (applies to both formats):
    * `root` (default): member closest to root (alphabetical tie-break).
    * `medoid`: member with min sum of distances to others (alphabetical tie-break).
    * `first`: alphabetically first member.
* `--dynamic-hybrid` requires a `--matrix` distance file.

Examples:

1. Cut into 5 clusters
   `necom clust cut tree.nwk --k 5`

2. Cut at height 0.5
   `necom clust cut tree.nwk --height 0.5`

3. Dynamic Tree Cut with min cluster size 20
   `necom clust cut tree.nwk --dynamic-tree 20`

4. Hybrid Cut with PAM
   `necom clust cut tree.nwk --dynamic-hybrid 20 --matrix dist.phy`
