Remove nodes from a Newick tree based on labels or patterns.

Input:

* A Newick tree file.

Notes:

* Target nodes can be specified by `--node`, `--name-list`, or `--regex`.
* With `--invert`, specified nodes (along with their ancestors and descendants) are kept, and everything else is removed.
* Topology changes:
    * If a node removal leaves its parent with only one child, the parent is collapsed.
    * Internal nodes that lose all children are also removed.

Examples:

1. Remove specific nodes by name
   `necom nwk prune input.nwk -n Homo -n Pan`

2. Remove nodes using a list in a file
   `necom nwk prune input.nwk -l remove.txt`

3. Keep a clade and remove everything else (invert mode)
   `necom nwk prune input.nwk -i -n Hominidae`
