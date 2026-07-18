Cluster entries using DBSCAN.

Input:

* A pairwise distance TSV file (`name1\tname2\tdistance`). Lower distances indicate higher similarity.

Output:

* `--format cluster` (default): each line contains points of one cluster.
* `--format pair`: each line contains a `(representative, member)` pair.

Notes:

* The input must contain distances, not similarities.
* `--eps <V>`: neighborhood radius (default: `0.05`).
* `--min-points <N>`: minimum number of points (including the point itself) to form a dense region (default: `4`).
* `--same <V>`: default score of identical element pairs (default: `0.0`).
* `--missing <V>`: default score of missing pairs (default: `1.0`).
* The representative point is selected by `--rep`:
    * `medoid` (default): point with minimum sum of distances to other cluster members.
    * `first`: alphabetically first member.
* In `cluster` format, the representative is placed first; in `pair` format, it is the first column.
* The neighborhood count used by `--min-points` includes the point itself (self-distance is 0, which is always <= eps).

Examples:

1. Run DBSCAN with defaults
   `necom clust dbscan pairs.tsv`

2. Set epsilon and min points
   `necom clust dbscan pairs.tsv --eps 0.05 --min-points 5`

3. Output as pairs
   `necom clust dbscan pairs.tsv --format pair`
