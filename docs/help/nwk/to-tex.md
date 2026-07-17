Convert a Newick tree to a complete LaTeX document.

Input:

* A Newick tree file.
* If the file contains multiple trees, only the first tree is processed.
* Use `--forest` to pass through a file that already contains Forest code.

Notes:

* Styles are stored in the comments of each node.
* Draws a cladogram by default.
* Use `--bl` to draw a phylogram with branch lengths.
* Underscores (`_`) in names, labels, and comments are replaced with spaces.
* Other LaTeX special characters (`{ } \ # $ % & ~ ^`) in names, labels, and comments are escaped automatically.
* Requires a LaTeX installation with `fontspec`, `xeCJK` (for East Asian characters), and the `forest` package. Compilation can be done with `tectonic` or `latexmk -xelatex`.
* Use `--no-default-style` to keep the template's original font setup instead of injecting the default `Noto Sans` configuration.

Examples:

1. Generate a LaTeX file
   `necom nwk to-tex tree.nwk -o tree.tex`

2. Compile with Tectonic
   `tectonic tree.tex`

3. Compile with Latexmk
   `latexmk -xelatex tree.tex`
