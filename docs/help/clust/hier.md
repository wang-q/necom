Perform agglomerative hierarchical clustering on a distance matrix.

Input:

* A PHYLIP format distance matrix (strict or relaxed).
* For pairwise list input (`name1\tname2\tdist`), use `necom mat to-phylip` first.

Output:

* A Newick tree string written to stdout or `--outfile`.

Notes:

* The output tree uses the linkage distance (merge height) as node height.
* `--method <METHOD>`: linkage method (default: `ward`). Supported: `single`, `complete`, `average` (`upgma`), `weighted` (`wpgma`), `centroid` (`upgmc`), `median` (`wpgmc`), `ward` (`ward.d2`).
* For reducible methods (`single`, `complete`, `average`, `weighted`, `ward`), the default NN-chain implementation may produce a different merge order than the primitive algorithm, but the resulting partition sequence is equivalent.
* `centroid` and `median` linkage may produce non-monotonic merge heights (inversions); branch lengths are clamped at `0.0` so the resulting Newick tree remains valid. The same clamp is applied to all methods as a safety net.
* For Ward's method, the input is assumed to be Euclidean distances or similar.
* Alias: `necom clust hclust` is equivalent to `necom clust hier`.

Examples:

1. Ward's method (default)
   `necom clust hier matrix.phy > tree.nwk`

2. Average linkage (UPGMA)
   `necom clust hier matrix.phy --method average > tree.nwk`

3. Single linkage (nearest neighbor)
   `necom clust hier matrix.phy --method single > tree.nwk`
