Construct a phylogenetic tree using Neighbor-Joining (NJ).

Input:

* A PHYLIP distance matrix (relaxed or strict).

Output:

* A Newick tree rooted at the midpoint of the final edge.

Notes:

* NJ is a bottom-up clustering method suitable for variable evolutionary rates.
* For non-additive distance matrices, negative branch lengths are clamped to 0.

Examples:

1. Build tree from matrix
   `necom clust nj matrix.phy -o tree.nwk`

2. Pipe matrix from stdin
   `cat matrix.phy | necom clust nj stdin > tree.nwk`
