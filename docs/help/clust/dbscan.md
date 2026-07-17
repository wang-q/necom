Cluster entries using DBSCAN.

Input:

* A pairwise distance TSV file (`name1\tname2\tdistance`). Lower distances indicate higher similarity.

Output:

* `--format cluster` (default): each line contains points of one cluster.
* `--format pair`: each line contains a `(representative, member)` pair.

Notes:

* The input must contain distances, not similarities.
* `--same <V>`: default score of identical element pairs (default: `0.0`).
* `--missing <V>`: default score of missing pairs (default: `1.0`).
* The representative point is selected by `--rep`:
    * `medoid` (default): point with minimum sum of distances to other cluster members.
    * `first`: alphabetically first member.
* In `cluster` format, the representative is placed first; in `pair` format, it is the first column.

Examples:

1. Run DBSCAN with defaults
   `necom clust dbscan pairs.tsv`

2. Set epsilon and min points
   `necom clust dbscan pairs.tsv --eps 0.05 --min-points 5`

3. Output as pairs
   `necom clust dbscan pairs.tsv --format pair`
