Cut a tree using Dynamic Tree Cut.

Input:

* A Newick file containing a single tree.
* Branch lengths are used to compute node heights.

Output:

* `--format cluster` (default): each line contains the members of one cluster; the first member is the representative.
* `--format pair`: each line contains a `(representative, member)` pair.

Notes:

* `--min-size` is required and specifies the minimum cluster size.
* `--deep-split` enables more aggressive splitting.
* `--max-tree-height` sets the maximum joining height (default: 99% of tree height).
* `--rep` selects the cluster representative: `root` (default), `first`, or `medoid`.
* `--support <S>` treats edges with support `< S` as effectively infinite length, forcing a cut at low-support positions.
* Leaves that fall below the minimum cluster size are labeled as unassigned (cluster `0`) and are still emitted.

Examples:

1. Dynamic Tree Cut with min cluster size 20
   `necom cut dynamic tree.nwk --min-size 20`

2. Enable deep split
   `necom cut dynamic tree.nwk --min-size 10 --deep-split`
