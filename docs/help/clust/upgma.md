Construct a phylogenetic tree using UPGMA.

Input:

* A PHYLIP distance matrix (relaxed or strict).

Output:

* A rooted Newick tree.

Notes:

* UPGMA assumes an ultrametric (molecular clock): all leaves are equidistant from the root.
* When the input distances violate ultrametricity, the tree is still produced; negative branch lengths are clamped to 0 so the output remains a valid Newick tree.
* For non-ultrametric data, consider `necom clust nj` (neighbor-joining) instead.

Examples:

1. Build tree from matrix
   `necom clust upgma matrix.phy -o tree.nwk`

2. Pipe matrix from stdin
   `cat matrix.phy | necom clust upgma stdin > tree.nwk`
