Scan DBSCAN epsilon values and report internal clustering metrics.

Description:

* Runs DBSCAN over a range of `eps` values while keeping `min-points` fixed.
* Outputs a TSV summary table with cluster count, noise count, Silhouette score, and Davies-Bouldin Index for each epsilon.
* Alternatively, use `--opt-eps` to select the best epsilon and output the corresponding partition.

Input:

* A pairwise distance TSV file (`name1\tname2\tdistance`). Lower distances indicate higher similarity.

Output:

* Without `--opt-eps`: a TSV table with columns `Epsilon`, `Clusters`, `Noise`, `Silhouette`, `DBIndex`.
* With `--opt-eps`: a clustering partition in `--format cluster` (default) or `--format pair`.

Notes:

* `--scan <start,end,step>`: required. Epsilon scan range. All values must be positive finite numbers and `start <= end`. The explicit `end` value is always included.
* `--min-points <N>`: minimum number of points (including the point itself) to form a dense region (default: `4`).
* `--min-pct <P>`: alternative to `--min-points`; specify the minimum as a fraction of the total number of samples (range `(0, 1]`). The effective value is `ceil(P * n_samples)`. Mutually exclusive with `--min-points`.
* `--same <V>`: default score of identical element pairs (default: `0.0`).
* `--missing <V>`: default score of missing pairs (default: `1.0`).
* `--opt-eps <criterion>`: select the best epsilon and output that partition instead of the summary.
    * `silhouette`: maximize the Silhouette score.
    * `max-clusters`: maximize the number of non-noise clusters.
    * `min-noise`: minimize the number of noise points.
    * Ties are resolved by choosing the smaller epsilon.
* The representative point for the output partition is selected by `--rep`:
    * `medoid` (default): point with minimum sum of distances to other cluster members.
    * `first`: alphabetically first member.
* In `cluster` format, the representative is placed first; in `pair` format, it is the first column.
* Noise points (points not assigned to any density cluster) are emitted as single-member clusters.
* Metrics are computed on the emitted partition, where each noise point forms its own singleton cluster. Non-finite metric values are emitted as `NA`.
* Silhouette and Davies-Bouldin are computed using the distance-matrix definitions from `necom eval partition --matrix`. See `docs/eval-partition.md` for detailed metric definitions.
* Performance: scanning costs `steps × O(N²)`. Reduce the range or step count for large matrices.

Examples:

1. Scan eps from 0.05 to 0.5 with 50 steps
   `necom clust scan-dbscan pairs.tsv --scan 0.05,0.5,0.01`

2. Scan and select the epsilon with the highest Silhouette
   `necom clust scan-dbscan pairs.tsv --scan 0.05,0.5,0.01 --opt-eps silhouette`

3. Use --min-pct instead of --min-points
   `necom clust scan-dbscan pairs.tsv --scan 0.05,0.5,0.01 --min-pct 0.1`

4. Output the best partition as pairs
   `necom clust scan-dbscan pairs.tsv --scan 0.05,0.5,0.01 --opt-eps max-clusters --format pair`
