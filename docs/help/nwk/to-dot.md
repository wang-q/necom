Convert Newick trees to Graphviz DOT format for visualization.

Input:

* A Newick tree file.

Output:

* Graphviz DOT format.

Notes:

* Render with Graphviz tools such as `dot`, `neato`, or `twopi` (e.g., `dot -Tpng tree.dot -o tree.png`).
* Node and edge styles (colors, labels) are derived from Newick comments added by `necom nwk comment`.

Examples:

1. Convert to DOT
   `necom nwk to-dot tree.nwk`

2. Save to file
   `necom nwk to-dot tree.nwk -o tree.dot`

3. Create an image (requires Graphviz)
   `necom nwk to-dot tree.nwk | dot -Tpng -o tree.png`
