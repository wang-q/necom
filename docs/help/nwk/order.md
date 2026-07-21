Sort the children of each node without changing the topology.

Input:

* A Newick tree file.

Notes:

* Visits every internal node and sorts its children according to the selected criterion; different modes use post-order or level-order traversals internally.
* When no sort criterion is specified, defaults to forward alphanumeric order (equivalent to `--alphanumeric`).
* `--name-list` is processed before `--alphanumeric` and `--num-descendants`.
* `--alphanumeric` and `--num-descendants` can be combined; sorting is first alphanumeric, then by number of descendants.
* Sort orders:
    * `--name-list`: by a list of names in the file, one per line.
    * `--alphanumeric` / `--alphanumeric-rev`: by alphanumeric order of labels, or in reverse order.
    * `--num-descendants` / `--num-descendants-rev`: by number of descendants (ladderize), or in reverse order.
    * `--deladderize` (alias `--dl`): alternate sort direction at each level.
    * `--olo <MATRIX>`: optimal leaf ordering using a distance matrix. This option cannot be combined with other sort orders.
* `--olo-format` controls the format of the `--olo` matrix: `phylip` (default) or `pair`.
* Entries in `--name-list` that are not found among the leaf names cause the command to fail with an error listing the missing names.
* The `--olo` matrix must cover every leaf in the tree; missing leaves cause an error. Non-finite distances are also rejected.
* `--olo` expects the original data-space distance matrix, not the tree's own path distances. Optimizing against tree-derived distances would mostly reproduce the current topology, defeating the purpose of leaf reordering.

Examples:

1. Sort by number of descendants (ladderize)
   `necom nwk order tree.nwk --num-descendants`

2. Sort by alphanumeric order
   `necom nwk order tree.nwk --alphanumeric`

3. Sort by a list of names
   `necom nwk order tree.nwk --name-list names.txt`

4. Sort by alphanumeric order, then by number of descendants (reverse)
   `necom nwk order tree.nwk --alphanumeric --num-descendants-rev`

5. De-ladderize
   `necom nwk order tree.nwk --deladderize`

6. Optimal leaf ordering with a PHYLIP distance matrix
   `necom nwk order tree.nwk --olo distances.phy`

7. Optimal leaf ordering with a pairwise TSV matrix
   `necom nwk order tree.nwk --olo pairs.tsv --olo-format pair`
