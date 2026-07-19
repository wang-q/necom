Assign support values to internal nodes of a target tree based on replicate trees.

Input:

* Target tree file (first positional argument). May contain one or more trees; each is annotated and emitted on its own line.
* Replicate trees file (second positional argument), e.g., from bootstrap or jackknife resampling.

Notes:

* Support values are written as internal node labels.
* All trees must share the same set of leaves; replicate trees are checked against the first replicate and target trees are checked against the replicate set.
* The root node is not annotated; any existing root label is preserved.
* `--percent` / `-p`: output support values as integer percentages (0–100), truncated toward zero.
* `--override-root` / `-r`: override the root node label with its support value.

Examples:

1. Attribute support values
   `necom eval replicate target.nwk replicates.nwk`

2. Output support as percentages
   `necom eval replicate target.nwk replicates.nwk --percent`

3. Override root label with support value
   `necom eval replicate target.nwk replicates.nwk --override-root`
