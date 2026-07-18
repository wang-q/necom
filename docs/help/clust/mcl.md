Cluster entries using Markov Clustering (MCL).

Input:

* A pairwise similarity TSV file (`name1\tname2\tsimilarity`). Higher scores indicate higher similarity.

Output:

* `--format cluster` (default): each line contains points of one cluster.
* `--format pair`: each line contains a `(representative, member)` pair.

Notes:

* The input must contain similarities, not distances.
* `--inflation <V>` / `-I`: inflation parameter (default: `2.0`).
* `--prune <V>`: pruning threshold; matrix entries smaller than this are set to zero (default: `1e-5`).
* `--max-iter <N>`: maximum number of iterations (default: `100`). Must be greater than 0.
* `--same <V>`: default score of identical element pairs (default: `1.0`).
* `--missing <V>`: default score of missing pairs (default: `0.0`).
* The representative point is selected by `--rep`:
    * `medoid` (default): point with maximum sum of similarities to other cluster members.
    * `first`: alphabetically first member.
* In `cluster` format, the representative is placed first; in `pair` format, it is the first column.
* During initialization, input similarities smaller than or equal to `1e-5` (including negative values) are treated as zero and ignored. This initial filtering threshold is independent of the `--prune` threshold used during iterations.
* Reference: Stijn van Dongen, Graph Clustering by Flow Simulation. PhD thesis, University of Utrecht, May 2000.

Examples:

1. Run MCL with defaults
   `necom clust mcl similarities.tsv`

2. Adjust inflation
   `necom clust mcl similarities.tsv --inflation 3.0`

3. Output as pairs
   `necom clust mcl similarities.tsv --format pair`
