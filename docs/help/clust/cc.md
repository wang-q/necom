Cluster entries via connected components.

Input:

* A pairwise TSV file (`name1\tname2\t[weight]`). Weights are ignored.

Output:

* `--format cluster` (default): each line contains all points of one cluster.
* `--format pair`: each line contains a `(representative, member)` pair.

Notes:

* In `pair` format, the representative is the alphabetically first member of the cluster.

Examples:

1. Find connected components
   `necom clust cc pairs.tsv`

2. Output as pairs
   `necom clust cc pairs.tsv --format pair`
