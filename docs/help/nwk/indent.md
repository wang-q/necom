Re-format a Newick tree to make its structure easier to read.

Input:

* A Newick tree file.

Notes:

* By default, prints the tree indented with two spaces.
* The default output is valid Newick.
* Use `--compact` to output the tree as a single line.
* Using non-whitespace characters for `--text` may produce invalid Newick.

Examples:

1. Default indentation
   `necom nwk indent tree.nwk`

2. Compact output
   `necom nwk indent tree.nwk --compact`

3. Indent with visual guides (not valid Newick)
   `necom nwk indent tree.nwk --text ".   "`
