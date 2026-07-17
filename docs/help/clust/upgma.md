Construct a phylogenetic tree using UPGMA.

Input:

* A PHYLIP distance matrix (relaxed or strict).

Output:

* A Newick tree.

Examples:

1. Build tree from matrix
   `necom clust upgma matrix.phy -o tree.nwk`

2. Pipe matrix from stdin
   `cat matrix.phy | necom clust upgma stdin > tree.nwk`
