Scan static threshold cut parameters.

Input:

* A Newick file containing a single tree.

Output:

* Always writes the long format table `Group\tClusterID\tSampleID`.
* If `--stats-out` is given, also writes a summary table with `Group\tClusters\tSingletons\tNon-Singletons\tMaxSize`.

Notes:

* `--method` and `--range` are required.
* `--range` format is `start,end,step` with comma separators and no spaces.
* `--deep` controls the depth for the `inconsistent` method (default: `2`).
* `--support <S>` treats edges with support `< S` as effectively infinite length.
* The `--format` and `--rep` options are not available in scan mode.

Examples:

1. Scan heights and save statistics
   `necom cut scan-simple tree.nwk --method height --range 0,1.0,0.05 --stats-out stats.tsv > partitions.tsv`

2. Scan max-clade thresholds
   `necom cut scan-simple tree.nwk --method max-clade --range 0,0.5,0.01`
