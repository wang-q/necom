Convert a Newick tree to SVG format for visualization.

Input:

* A Newick tree file.
* If the file contains multiple trees, only the first tree is processed.

Notes:

* Automatically draws a phylogram if branch lengths are present; otherwise draws a cladogram.
* Underscores (`_`) in names are replaced with spaces.
* Default styles match the LaTeX Forest template (grey branches, black dots).
* A scale bar is drawn in phylogram mode.
* `--width` / `-w`: SVG canvas width in pixels (default: `800`). Must be a positive finite number.
* `--vskip` / `-v`: vertical spacing between leaf nodes in pixels (default: `20`). Must be a positive finite number.

Examples:

1. Convert to SVG
   `necom nwk to-svg tree.nwk -o tree.svg`

2. Custom width and spacing
   `necom nwk to-svg tree.nwk -w 1200 -v 30 -o tree.svg`
