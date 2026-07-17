Print statistics about trees in the input.

Input:

* A Newick tree file or `stdin`.

Output:

* `--style col` (default): key-value pairs.
* `--style line`: tab-separated values with a header row.

Reported fields include type (cladogram/phylogram/neither), node count, leaf count, rooted status, dichotomies, leaf labels, internal labels, edges with/without length, cherries, Sackin index, and Colless index.

Examples:

1. Default statistics
   `necom nwk stat tree.nwk`

2. Output to file
   `necom nwk stat tree.nwk -o stats.tsv`
