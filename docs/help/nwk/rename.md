Rename nodes in a Newick tree.

Input:

* A Newick tree file.

Notes:

* For nodes with names, use `--node`.
* For unnamed internal nodes, use `--lca` with two comma-separated names.
* The total number of `--node` and `--lca` arguments must equal the number of `--rename` (`-r`) arguments.
* Matching order: the first N `--rename` values are applied to the N `--node` arguments in order, and the remaining `--rename` values are applied to the `--lca` arguments in order. The order in which `--node`, `--lca`, and `--rename` arguments appear on the command line does not affect this matching.
* This command is designed for small edits, not batch replacement. For batch replacement, use `necom nwk replace` or external tools such as `sed` or `perl`.

Examples:

1. Rename a named node
   `necom nwk rename tree.nwk --node Homo --rename Human`

2. Rename an internal node via LCA
   `necom nwk rename tree.nwk --lca Homo,Pan --rename CladeX`

3. Rename multiple nodes
   `necom nwk rename tree.nwk \
       --node Homo --rename Human \
       --lca Homo,Pan --rename CladeX`
