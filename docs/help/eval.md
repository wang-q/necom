Evaluate clustering partitions and phylogenetic trees.

Input:

* `eval` is a command group; use one of its subcommands directly.

Notes:

* Subcommand groups:
    * Tree comparison: `compare`
    * Partition evaluation: `partition`
    * Branch support: `replicate`
* Run `necom eval <subcommand> --help` for command-specific options.
* `eval compare` reads from stdin if the input file is `stdin`.
* `eval partition` accepts `stdin` for `p1` and for `--other`/`--matrix`/`--tree`/`--coords`.
* `eval replicate` accepts `stdin` for the target or replicates file (but not both at once).

Examples:

1. Show available subcommands
   `necom eval --help`
