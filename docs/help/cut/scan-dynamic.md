Scan Dynamic Tree Cut min cluster sizes.

Input:

* A Newick file containing a single tree.

Output:

* Always writes the long format table `Group\tClusterID\tSampleID`.
* If `--stats-out` is given, also writes a summary table with `Group\tClusters\tSingletons\tNon-Singletons\tMaxSize`.

Notes:

* The `--range` option is required and must be `start,end,step` with comma separators and no spaces.
* All three range values must be non-negative integers and the step must be positive.
* `--deep-split` enables more aggressive splitting (default: off).
* `--max-tree-height` sets the maximum joining height; if omitted, 99% of the tree height is used.
* `--support <S>` treats edges with support `< S` as effectively infinite length.
* The `--format` and `--rep` options are not available in scan mode.
* The output `Group` column is labeled `dynamic-tree=<min-size>`.

Examples:

1. Scan min cluster sizes from 2 to 20
   `necom cut scan-dynamic tree.nwk --range 2,20,2 --stats-out stats.tsv > partitions.tsv`

2. Scan with deep split enabled
   `necom cut scan-dynamic tree.nwk --range 2,20,2 --deep-split --stats-out stats.tsv > partitions.tsv`
