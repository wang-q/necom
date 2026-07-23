Sort the children of each node without changing the topology.

Input:

* A Newick tree file.

Notes:

* Visits every internal node and sorts its children according to the selected criterion; different modes use post-order or level-order traversals internally.
* When no sort criterion is specified, defaults to forward alphanumeric order (equivalent to `--alphanumeric`).
* Multiple sort options can be given in one command; they are applied in this order:
    1. `--name-list`
    2. `--alphanumeric` / `--alphanumeric-rev`
    3. `--num-descendants` / `--num-descendants-rev`
    4. `--deladderize`
  Each step fully reorders the children of every internal node, so later steps override earlier ones.
* `--olo` cannot be combined with other sort options because it computes a global optimal order from a distance matrix.
* To combine criteria as tie-breakers instead of sequential overrides, pipe multiple `nwk order` calls:
   `necom nwk order tree.nwk --num-descendants | necom nwk order stdin --alphanumeric`
* Sort orders:
    * `--name-list`: by a list of names in the file, one per line.
    * `--alphanumeric` (`--an`) / `--alphanumeric-rev` (`--anr`): by alphanumeric order of labels, or in reverse order.
    * `--num-descendants` (`--nd`) / `--num-descendants-rev` (`--ndr`): by number of descendants (ladderize), or in reverse order.
    * `--deladderize` (alias `--dl`): alternate sort direction at each level.
    * `--olo <MATRIX>`: optimal leaf ordering using a distance matrix.
* `--olo-format` controls the format of the `--olo` matrix: `phylip` (default) or `pair`.
* `--olo` expects the original data-space distance matrix, not the tree's own path distances. Optimizing against tree-derived distances would mostly reproduce the current topology, defeating the purpose of leaf reordering.
* Entries in `--name-list` that are not found among the leaf names cause the command to fail with an error listing the missing names.
* The `--olo` matrix must cover every leaf in the tree; missing leaves cause an error. Non-finite distances are also rejected.

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
