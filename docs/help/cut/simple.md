Cut a tree using a static threshold method.

Input:

* A Newick file containing a single tree.
* Branch lengths are used by distance/height-based methods.

Output:

* `--format cluster` (default): each line contains the members of one cluster; the first member is the representative.
* `--format pair`: each line contains a `(representative, member)` pair.

Notes:

* `--method` is required. Accepted values: `k`, `height`, `root-dist`, `max-clade`, `avg-clade`, `med-clade`, `sum-branch`, `leaf-dist-max`, `leaf-dist-min`, `leaf-dist-avg`, `max-edge`, `inconsistent`.
* `--threshold` is required. For `--method k` it must be a positive integer.
* `--deep` controls the depth used by the `inconsistent` method (default: `2`).
* `--rep` selects the cluster representative: `root` (default), `first`, or `medoid`.
* `--support <S>` treats edges with support `< S` as effectively infinite length, forcing a cut at low-support positions. Nodes without explicit support default to `100.0`.
* Distance/height thresholds must be non-negative finite numbers.

Examples:

1. Cut into 5 clusters
   `necom cut simple tree.nwk --method k --threshold 5`

2. Cut at height 0.5
   `necom cut simple tree.nwk --method height --threshold 0.5`

3. Cut by max pairwise distance
   `necom cut simple tree.nwk --method max-clade --threshold 0.1`

4. Use pair format
   `necom cut simple tree.nwk --method k --threshold 3 --format pair`
