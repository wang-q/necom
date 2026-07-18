Cluster entries via various algorithms.

Input:

* `clust` is a command group; use one of its subcommands directly.

Notes:

* Subcommand groups:
    * Tree: `hier`, `nj`, `upgma`
    * Flat: `cc`, `dbscan`, `k-medoids`, `mcl`
* Run `necom clust <subcommand> --help` for algorithm-specific options.

Examples:

1. Show available subcommands
   `necom clust --help`
