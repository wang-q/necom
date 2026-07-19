Cut a tree using Dynamic Hybrid Cut.

Input:

* A Newick file containing a single tree.
* A distance matrix file in relaxed PHYLIP format (`--matrix`).

Output:

* `--format cluster` (default): each line contains the members of one cluster; the first member is the representative.
* `--format pair`: each line contains a `(representative, member)` pair.

Notes:

* `--matrix` and `--min-size` are required.
* `--max-pam-dist` limits the distance for PAM reassignment.
* `--no-pam-dendro` disables dendrogram respect during PAM reassignment.
* `--deep-split` enables more aggressive splitting.
* `--max-tree-height` sets the maximum joining height (default: 99% of tree height).
* `--rep` selects the cluster representative: `root` (default), `first`, or `medoid`.
* `--support <S>` treats edges with support `< S` as effectively infinite length, forcing a cut at low-support positions.

Examples:

1. Hybrid cut with min cluster size 20
   `necom cut hybrid tree.nwk --matrix dist.phy --min-size 20`

2. Allow PAM reassignment across high branches
   `necom cut hybrid tree.nwk --matrix dist.phy --min-size 10 --no-pam-dendro`
