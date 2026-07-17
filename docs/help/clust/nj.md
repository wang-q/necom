Construct a phylogenetic tree using Neighbor-Joining (NJ).

Input:

* A PHYLIP distance matrix (relaxed or strict).

Output:

* A midpoint-rooted Newick tree.

Notes:

* NJ is a bottom-up clustering method suitable for variable evolutionary rates.

Examples:

1. Build tree from matrix
   `necom clust nj matrix.phy -o tree.nwk`

2. Pipe matrix from stdin
   `cat matrix.phy | necom clust nj stdin > tree.nwk`
