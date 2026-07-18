Calculate distances between nodes or generate distance matrices.

Input:

* A valid Newick tree file.
* If the file contains multiple trees, only the first tree is processed.

Notes:

* Modes (`--mode`):
    * `root` (default): distance from each node to the root. Output: `Node \t Distance`.
    * `parent`: distance from each node to its parent. Output: `Node \t Distance`.
    * `pairwise`: distance between every pair of selected nodes, including self-pairs and both `(i, j)` and `(j, i)` orderings. Output: `Node1 \t Node2 \t Distance`.
    * `lca`: distance from each node in a pair to their Lowest Common Ancestor (LCA), for all selected-node pairs. Output: `Node1 \t Node2 \t Dist1 \t Dist2`.
    * `phylip`: a PHYLIP-formatted distance matrix for the selected nodes.
* Use `-I` to exclude internal nodes and `-L` to exclude leaf nodes.
* Use `-n` / `-l` / `-x` to restrict reported nodes to a name, name-list file, or regex.
* When no name-based filter is given, all selected nodes (respecting `-I`/`-L`) are reported.
* `--mode phylip` requires all selected nodes to be named; node names cannot contain whitespace characters.

Examples:

1. Distances to root (default)
   `necom nwk distance tree.nwk`

2. Pairwise distances
   `necom nwk distance tree.nwk --mode pairwise`

3. Generate a PHYLIP matrix
   `necom nwk distance tree.nwk --mode phylip > matrix.phy`

4. Distances to parent for leaves only
   `necom nwk distance tree.nwk --mode parent -I`

5. Distance to root for selected nodes
   `necom nwk distance tree.nwk --mode root -n Homo -n Pan`
