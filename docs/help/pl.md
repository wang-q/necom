Run integrated pipelines.

Input:

* `pl` is a command group; use one of its subcommands directly.

Notes:

* Currently implemented subcommand:
    * `condense`: condense monophyletic subtrees based on a taxonomy TSV file.
* Run `necom pl <subcommand> --help` for command-specific options.
* Reads from stdin if input file is 'stdin'.

Examples:

1. Show available subcommands
   `necom pl --help`
