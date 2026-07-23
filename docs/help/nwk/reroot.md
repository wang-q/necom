Reroot a phylogenetic tree on a specific branch or node.

Input:

* A Newick tree file.
* If the file contains multiple trees, only the first tree is processed.

Notes:

* Target selection:
    * Default (no nodes specified): reroot at the midpoint of the longest branch.
    * `--node` / `-n`: reroot on the edge leading to the LCA of the specified nodes. The specified nodes become the ingroup.
    * `--lax` / `-l`: if the LCA of specified nodes is already the root, use the unspecified nodes as the ingroup instead. Useful for defining an outgroup by exclusion.
* Operations:
    * Reroot (default): creates a bifurcating root at the target edge.
    * `--deroot` / `-d`: converts a bifurcating root into a multifurcating root by collapsing its internal children into the root. Takes priority over `--node` and `--lax` (which are ignored when `--deroot` is given).
* `--support-as-labels` / `-s`: treat internal node labels as support values and shift them along the rerooting path to maintain split associations.
* Topology cleanup: the original root's parent edge is merged, and degree-2 nodes created during the process are removed.

Examples:

1. Reroot at the longest branch (default)
   `necom nwk reroot input.nwk`

2. Reroot at a specific node
   `necom nwk reroot input.nwk -n Homo`

3. Reroot at the LCA of multiple nodes
   `necom nwk reroot input.nwk -n Homo -n Pan`

4. Reroot and preserve support values
   `necom nwk reroot input.nwk -n Homo --support-as-labels`
