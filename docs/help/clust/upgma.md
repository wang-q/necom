Construct a phylogenetic tree using UPGMA.

Input:

* A PHYLIP distance matrix (relaxed or strict).

Output:

* A Newick tree.

Notes:

* UPGMA assumes an ultrametric (molecular clock): all leaves are equidistant from the root.
* When the input distances violate ultrametricity, the tree is still produced but branch lengths may not reflect true evolutionary distances.
* For non-ultrametric data, consider `necom clust nj` (neighbor-joining) instead.

Examples:

1. Build tree from matrix
   `necom clust upgma matrix.phy -o tree.nwk`

2. Pipe matrix from stdin
   `cat matrix.phy | necom clust upgma stdin > tree.nwk`
