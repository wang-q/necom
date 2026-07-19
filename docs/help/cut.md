Cut a Newick tree into flat clustering partitions.

Input:

* A Newick tree file containing a single tree.
* Branch lengths are used by distance/height-based methods.
* Branch support values (optional) can be used as a non-crossable constraint via `--support`.

Output:

* `--format cluster` (default): each line contains points of one cluster; the first point is the representative.
* `--format pair`: each line contains a `(representative, member)` pair.

Notes:

* Cutting methods (mutually exclusive):
    * `--k <K>`: cut into K clusters (top-down split by height).
    * `--height <H>`: cut at specific height (max distance to leaves).
    * `--root-dist <D>`: cut at specific distance from root.
    * `--max-clade <T>`: TreeCluster style (max pairwise distance in clade <= T).
    * `--avg-clade <T>`: TreeCluster style (avg pairwise distance in clade <= T).
    * `--med-clade <T>`: TreeCluster style (median pairwise distance in clade <= T).
    * `--sum-branch <T>`: TreeCluster style (sum of branch lengths in clade <= T).
    * `--leaf-dist-max <T>`: TreeCluster style (max distance from cluster root to any leaf <= T). Approximate: uses global leaf depth statistics converted to a root-distance cut.
    * `--leaf-dist-min <T>`: TreeCluster style (min distance from cluster root to any leaf <= T). Approximate: same conversion as `--leaf-dist-max`.
    * `--leaf-dist-avg <T>`: TreeCluster style (avg distance from cluster root to leaves <= T). Approximate: same conversion as `--leaf-dist-max`.
    * `--max-edge <T>` / `--single-linkage <T>`: cut branches longer than threshold.
    * `--inconsistent <T>`: SciPy style (inconsistent coefficient <= T).
    * `--dynamic-tree <N>`: Dynamic Tree Cut (top-down adaptive, N=min cluster size).
    * `--dynamic-hybrid <N>`: Hybrid Cut (Dynamic Tree + PAM, N=min cluster size).
* `--deep <N>`: depth for inconsistent coefficient calculation (default: `2`). Used with `--inconsistent`.
* `--max-pam-dist <D>`: maximum distance to medoid for PAM reassignment. Used with `--dynamic-hybrid`.
* `--no-pam-dendro`: disable dendrogram respect in PAM stage (allow assigning to clusters across high branches). Used with `--dynamic-hybrid`.
* `--deep-split`: enable deep split for dynamic tree cut (default: false). More aggressive splitting. Used with `--dynamic-tree` / `--dynamic-hybrid`.
* `--max-tree-height <H>`: maximum joining height for dynamic tree cut (default: 99% of tree height). Used with `--dynamic-tree` / `--dynamic-hybrid`.
* The representative is selected by `--rep`:
    * `root` (default): member closest to root (alphabetical tie-break).
    * `medoid`: member with the smallest sum of distances to other members.
    * `first`: alphabetically first member.
* `--support <S>` treats edges with support `< S` as effectively infinite length, forcing a cut. Nodes without explicit support default to `100.0`. The mask is applied before any cut method runs, so it affects all distance/height-based methods uniformly.
* `--scan <start>,<end>,<step>` performs a parameter sweep. In scan mode, `--format` is ignored and output is always long format (`Group\tClusterID\tSampleID`). For `--dynamic-tree`, the scan value replaces the min cluster size at each step (the `--dynamic-tree` argument itself only selects the method).
* `--stats-out <FILE>` writes scan summary statistics (`Group\tClusters\tSingletons\tNon-Singletons\tMaxSize`).
* `--dynamic-hybrid` requires `--matrix`.
* `--dynamic-tree` / `--dynamic-hybrid` may label leaves that fall below the minimum cluster size as unassigned (cluster `0`). Unassigned leaves are still emitted in the default output.
* In `--scan` output, cluster `0` keeps the label `0`; non-zero clusters are renumbered starting from `1`. The `--stats-out` `Clusters` column counts only non-zero clusters.
* Distance/height thresholds (`--height`, `--root-dist`, `--max-clade`, `--avg-clade`, `--med-clade`, `--sum-branch`, `--leaf-dist-*`, `--max-edge`, `--inconsistent`) must be non-negative.

Examples:

1. Cut into 5 clusters
   `necom cut tree.nwk --k 5`

2. Cut at height 0.5
   `necom cut tree.nwk --height 0.5`

3. Dynamic Tree Cut with min cluster size 20
   `necom cut tree.nwk --dynamic-tree 20`

4. Scan thresholds and save statistics
   `necom cut tree.nwk --max-clade 0.5 --scan 0,0.5,0.01 -o partitions.tsv --stats-out stats.tsv`
