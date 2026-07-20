Scan static threshold cut parameters.

Input:

* A Newick file containing a single tree.

Output:

* Always writes the long format table `Group\tClusterID\tSampleID`.
* If `--stats-out` is given, also writes a summary table with `Group\tClusters\tSingletons\tNon-Singletons\tMaxSize`.

Notes:

* Exactly one method flag must be provided: `--k`, `--height`, `--root-dist`, `--max-clade`, `--avg-clade`, `--med-clade`, `--sum-branch`, `--leaf-dist-max`, `--leaf-dist-min`, `--leaf-dist-avg`, `--max-edge` (`--single-linkage`), `--inconsistent`.
* `--range` is required and must be `start,end,step` with comma separators and no spaces.
* `--k --range` requires integer values and the start must be at least `1`.
* `--deep` controls the depth for the `inconsistent` method (default: `2`).
* Distance/height thresholds must be non-negative finite numbers.
* `--support <S>` treats edges with support `< S` as effectively infinite length.
* The `--format` and `--rep` options are not available in scan mode.

Examples:

1. Scan heights and save statistics
   `necom cut scan-simple tree.nwk --height --range 0,1.0,0.05 --stats-out stats.tsv > partitions.tsv`

2. Scan max-clade thresholds
   `necom cut scan-simple tree.nwk --max-clade --range 0,0.5,0.01`

3. Scan cluster counts
   `necom cut scan-simple tree.nwk --k --range 1,5,1`
