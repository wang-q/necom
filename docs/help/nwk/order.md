Sort the children of each node without changing the topology.

Input:

* A Newick tree file.

Notes:

* Visits every internal node and sorts its children according to the selected criterion; different modes use post-order or level-order traversals internally.
* `--name-list` is processed before `--alphanumeric` and `--num-descendants`.
* `--alphanumeric` and `--num-descendants` can be combined; sorting is first alphanumeric, then by number of descendants.
* Sort orders:
    * `--name-list`: by a list of names in the file, one per line.
    * `--alphanumeric` / `--alphanumeric-rev`: by alphanumeric order of labels.
    * `--num-descendants` / `--num-descendants-rev`: by number of descendants (ladderize).
    * `--deladderize` (alias `--dl`): alternate sort direction at each level.
* Entries in `--name-list` that are not found among the leaf names are logged as warnings and skipped.

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
