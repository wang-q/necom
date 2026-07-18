Manipulate, analyze, and visualize Newick trees.

Input:

* A Newick tree file or `stdin`.

Notes:

* Subcommand groups:
    * Information: `stat`, `label`, `distance`
    * Manipulation: `order`, `prune`, `rename`, `replace`, `reroot`, `subtree`, `topo`
    * Visualization: `comment`, `indent`, `to-dot`, `to-forest`, `to-svg`, `to-tex`
* Run `necom nwk <subcommand> --help` for command-specific options.
* Reads from stdin if input file is 'stdin'.

Examples:

1. Show available subcommands
   `necom nwk --help`
