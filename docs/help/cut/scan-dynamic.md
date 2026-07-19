Scan Dynamic Tree Cut min cluster sizes.

Input:

* A Newick file containing a single tree.

Output:

* Always writes the long format table `Group\tClusterID\tSampleID`.
* If `--stats-out` is given, also writes a summary table with `Group\tClusters\tSingletons\tNon-Singletons\tMaxSize`.

Notes:

* `--range` is required and all three values must be non-negative integers.
* `--range` format is `start,end,step` with comma separators and no spaces.
* `--deep-split` enables more aggressive splitting.
* `--max-tree-height` sets the maximum joining height (default: 99% of tree height).
* `--support <S>` treats edges with support `< S` as effectively infinite length.
* The `--format` and `--rep` options are not available in scan mode.

Examples:

1. Scan min cluster sizes from 2 to 20
   `necom cut scan-dynamic tree.nwk --range 2,20,2 --stats-out stats.tsv > partitions.tsv`
