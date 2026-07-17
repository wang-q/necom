Extract labels (names) from a Newick file.

Input:

* A Newick tree file.

Notes:

* By default, prints all non-empty labels in Newick order, one per line.
* Use `--tab` to print labels on a single line separated by tabs.
* Use `-I` to exclude internal nodes and `-L` to exclude leaf nodes.
* Selection options (`-n`, `-l`, `-x`) can be combined.
* With `-D`, descendants of selected internal nodes are also included.
* `-M` verifies that the selected nodes form a clade with at least two nodes.
* Duplicate node names may affect selection and clade checks.
* Extra columns (`-c`):
    * `dup`: duplicate the node name.
    * `taxid`: `:T=` field from the comment.
    * `species`: `:S=` field from the comment.
    * `full`: full comment.

Examples:

1. List all labels
   `necom nwk label tree.nwk`

2. Count leaves
   `necom nwk label tree.nwk -I | wc -l`

3. List specific nodes
   `necom nwk label tree.nwk -n Human -n Chimp`

4. List labels matching a regex
   `necom nwk label tree.nwk -x "^Homo"`

5. Check clade
   `necom nwk label tree.nwk -n Human -n Chimp -M`
