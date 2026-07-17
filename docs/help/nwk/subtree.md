Extract a subtree (clade) rooted at the lowest common ancestor (LCA) of the selected nodes.

Input:

* A Newick tree file.

Notes:

* Node selection: use `-n` for exact names, `-l` for a name-list file, or `-x` for a regular expression. If no selection is provided, no output is generated.
* `-M`: ensure the selected nodes form a clade with at least two terminal nodes.
* `--condense` / `-C`: instead of extracting the subtree, replace it with a single node in the original tree. The new node inherits the edge length of the subtree root and receives annotations `member=<count>` and `tri=white`.
* `--context` / `-c`: extend the subtree by N levels above the LCA.

Examples:

1. Extract a subtree for two nodes
   `necom nwk subtree tree.nwk -n Human -n Chimp`

2. Extract a subtree for nodes matching a regex
   `necom nwk subtree tree.nwk -x "^Homo"`

3. Condense a clade into a single node
   `necom nwk subtree tree.nwk -n Homo -n Pan --condense Hominini`

4. Check if a group is a clade
   `necom nwk subtree tree.nwk -n Human -n Chimp -n Gorilla -M`
