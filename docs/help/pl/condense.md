Condense monophyletic subtrees whose leaves share the same taxonomic term at a selected rank into a single node.

Input:

* `<infile>`: Newick tree whose leaf labels match the first column of the taxonomy TSV. Use `stdin` for standard input.
* `--taxon <taxon.tsv>`: Tab-separated file without header, containing at least 2 columns:
    * Column 1: node name (must match leaf labels in the Newick file).
    * Column 2+: taxonomic terms (e.g., species, genus, family).

Output:

* Condensed Newick tree written to `-o`/`--outfile` or `stdout`.
* With `--map`, also writes `condensed.tsv` to the current working directory. It contains two tab-separated columns: original node name and condensed label.

Notes:

* Use `--rank` to select which column(s) of the taxonomy TSV to use for grouping (1-based index, default: `2`).
* Multiple `--rank` values may be supplied to condense at multiple levels.
* Only monophyletic subtrees with the same taxonomic term are condensed; non-monophyletic groups are skipped.
* Condensed nodes are named `{term}||{count}`, where `{count}` is the number of leaves in the condensed group.
* Condensed nodes carry `member=<count>` and `tri=white` comments for visualization.

Examples:

1. Condense by species (2nd column)
   `necom pl condense --taxon taxon.tsv tree.nwk`

2. Condense by genus (3rd column)
   `necom pl condense --taxon taxon.tsv --rank 3 tree.nwk`

3. Condense by multiple ranks
   `necom pl condense --taxon taxon.tsv --rank 2 --rank 3 tree.nwk`

4. Output a mapping file alongside the condensed tree
   `necom pl condense --taxon taxon.tsv --map tree.nwk -o condensed.nwk`