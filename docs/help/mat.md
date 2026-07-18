Operate on distance matrices.

Input:

* `mat` is a command group; use one of its subcommands directly.

Notes:

* Subcommands:
    * `compare`: compare two PHYLIP distance matrices.
    * `format`: convert between PHYLIP matrix variants.
    * `from-vector`: calculate pairwise similarity/distance from vector inputs.
    * `subset`: extract a submatrix using a name list.
    * `to-pair`: flatten a PHYLIP matrix to pairwise TSV.
    * `to-phylip`: assemble pairwise TSV into a PHYLIP matrix.
    * `transform`: apply mathematical transformations to matrix elements.
* Run `necom mat <subcommand> --help` for command-specific options.
* Reads from stdin if input file is 'stdin'.

Examples:

1. Show available subcommands
   `necom mat --help`
