Cut a tree using Dynamic Hybrid Cut.

Input:

* A Newick file containing a single tree.
* A distance matrix file in relaxed PHYLIP format (`--matrix`).

Output:

* `--format cluster` (default): each line contains the members of one cluster; the first member is the representative.
* `--format pair`: each line contains a `(representative, member)` pair.

Notes:

* The `--matrix` and `--min-size` options are required. `--min-size` must be a positive integer.
* `--max-pam-dist` sets the maximum distance for PAM reassignment (optional).
* `--no-pam-dendro` disables dendrogram respect during PAM reassignment (default: off).
* `--deep-split` enables more aggressive splitting (default: off).
* `--max-tree-height` sets the maximum joining height; if omitted, defaults to `ref_height + 0.99 * (max_height - ref_height)`, where `ref_height` is the 5th percentile of merge heights and `max_height` is the maximum merge height.
* `--rep` selects the cluster representative: `root` (default), `first`, or `medoid`.
* `--support <S>` treats edges with support `< S` as effectively infinite length, forcing a cut at low-support positions.

Examples:

1. Hybrid cut with min cluster size 20
   `necom cut hybrid tree.nwk --matrix dist.phy --min-size 20`

2. Allow PAM reassignment across high branches
   `necom cut hybrid tree.nwk --matrix dist.phy --min-size 10 --no-pam-dendro`

3. Limit PAM reassignment distance
   `necom cut hybrid tree.nwk --matrix dist.phy --min-size 10 --max-pam-dist 0.3`
