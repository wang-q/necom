Perform agglomerative hierarchical clustering on a distance matrix.

Input:

* A PHYLIP format distance matrix (strict or relaxed).
* For pairwise list input (`name1 name2 dist`), use `necom mat to-phylip` first.

Output:

* A Newick tree string written to stdout or `--outfile`.

Notes:

* The output tree uses the linkage distance (merge height) as node height.
* `--method <METHOD>`: linkage method (default: `ward`). Supported: `single`, `complete`, `average`, `weighted`, `centroid`, `median`, `ward`.
* For Ward's method, the input is assumed to be Euclidean distances or similar.
* Alias: `necom clust hclust` is equivalent to `necom clust hier`.

Examples:

1. Ward's method (default)
   `necom clust hier matrix.phy > tree.nwk`

2. Average linkage (UPGMA)
   `necom clust hier matrix.phy --method average > tree.nwk`

3. Single linkage (nearest neighbor)
   `necom clust hier matrix.phy --method single > tree.nwk`
