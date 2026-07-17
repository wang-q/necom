Assign bootstrap support values to internal nodes of a target tree based on replicate trees.

Input:

* Target tree file (first positional argument).
* Replicate trees file (second positional argument), e.g., from bootstrap.

Notes:

* Support values are written as internal node labels.
* All trees must share the same set of leaves.
* The root node is not annotated; any existing root label is preserved.
* `--percent` / `-p`: output support values as integer percentages (0–100), truncated toward zero.

Examples:

1. Attribute support values
   `necom nwk support target.nwk replicates.nwk`

2. Output support as percentages
   `necom nwk support target.nwk replicates.nwk --percent`
