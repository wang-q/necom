Modify tree topology by optionally removing branch lengths, comments, or labels.

Input:

* A Newick tree file.

Notes:

* By default, branch lengths and comments are removed.
* Use `--bl` to keep branch lengths.
* Use `--comment` / `-c` to keep comments.
* Use `-I` to remove internal labels.
* Use `-L` to remove leaf labels.

Examples:

1. Topology only (remove lengths and comments)
   `necom nwk topo tree.nwk`

2. Keep branch lengths but remove comments
   `necom nwk topo tree.nwk --bl`

3. Remove internal node labels
   `necom nwk topo tree.nwk -I`
