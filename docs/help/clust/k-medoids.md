Cluster entries using K-Medoids (PAM / Lloyd-like).

Input:

* A pairwise distance TSV file (`name1\tname2\tdistance`). Lower distances indicate higher similarity.

Output:

* `--format cluster` (default): each line contains points of one cluster.
* `--format pair`: each line contains a `(representative, member)` pair.

Notes:

* The input must contain distances, not similarities.
* The representative point is selected by `--rep`:
    * `medoid` (default): point with minimum sum of distances to other cluster members.
    * `first`: alphabetically first member.
* In `cluster` format, the representative is placed first; in `pair` format, it is the first column.

Examples:

1. Run K-Medoids with K=3
   `necom clust k-medoids pairs.tsv --k 3`

2. Output as pairs
   `necom clust k-medoids pairs.tsv --k 3 --format pair`

3. Reproducible run with seed
   `necom clust k-medoids pairs.tsv --k 3 --seed 42`
