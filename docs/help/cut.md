Cut a Newick tree into flat clustering partitions.

`necom cut` is split into five focused subcommands. Run `necom cut <subcommand> --help` for detailed options.

Input:

* A Newick file containing a single tree.
* Branch lengths are used by distance/height-based methods.
* Branch support values (optional) can be used as a non-crossable constraint via `--support`.

Subcommands:

* `simple`: static threshold methods such as `k`, `height`, `max-clade`, `avg-clade`, `inconsistent`, etc.
* `dynamic`: Dynamic Tree Cut (top-down adaptive, `--min-size`).
* `hybrid`: Dynamic Hybrid Cut (tree + distance matrix, `--min-size`, `--matrix`).
* `scan-simple`: parameter sweep over a static method threshold.
* `scan-dynamic`: parameter sweep over dynamic-tree min cluster sizes.

Examples:

1. Cut into 5 clusters
   `necom cut simple tree.nwk --k 5`

2. Cut at height 0.5
   `necom cut simple tree.nwk --height 0.5`

3. Dynamic Tree Cut with min cluster size 20
   `necom cut dynamic tree.nwk --min-size 20`

4. Scan thresholds and save statistics
   `necom cut scan-simple tree.nwk --max-clade --range 0,0.5,0.01`