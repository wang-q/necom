Convert Newick trees to Graphviz DOT format for visualization.

Input:

* A Newick tree file.

Output:

* Graphviz DOT format.

Notes:

* Render with Graphviz tools such as `dot`, `neato`, or `twopi` (e.g., `dot -Tpng tree.dot -o tree.png`).
* Node labels come from Newick node names; edge labels come from positive branch lengths. Styling information from `necom nwk comment` annotations is not rendered.

Examples:

1. Convert to DOT
   `necom nwk to-dot tree.nwk`

2. Save to file
   `necom nwk to-dot tree.nwk -o tree.dot`

3. Create an image (requires Graphviz)
   `necom nwk to-dot tree.nwk | dot -Tpng -o tree.png`
