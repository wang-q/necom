Convert a Newick tree to raw LaTeX Forest code.

Input:

* A Newick tree file.
* If the file contains multiple trees, only the first tree is processed.

Notes:

* Styles are stored in the comments of each node.
* Draws a cladogram by default.
* Use `--bl` to draw a phylogram with branch lengths.
*   LaTeX special characters (`{ } \ # $ % & ~ ^`) and underscores in node names, labels, comments, and visualization attributes (`dot`, `bar`, `rec`, `tri`) are escaped or normalized for safe Forest output.

Examples:

1. Convert to Forest code
   `necom nwk to-forest tree.nwk`

2. Convert with branch lengths
   `necom nwk to-forest tree.nwk --bl`

3. Save to file
   `necom nwk to-forest tree.nwk -o forest.tex`
