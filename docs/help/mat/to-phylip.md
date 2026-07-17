Convert pairwise distances to a PHYLIP distance matrix.

Input:

* Tab-separated values (TSV).
* Three columns: name1, name2, distance.

Output:

* A full PHYLIP distance matrix.
* All observed IDs are collected and arranged into a square matrix.

Notes:

* Typical input comes from alignment results (e.g., `blast --outfmt 6`).
* Use `--same` to set the distance for self-to-self pairs (diagonal), default 0.0.
* Use `--missing` to set the distance for pairs without input records, default 1.0.
* The default missing value represents maximum distance, which is useful for unaligned sequences.

Examples:

1. Convert pairwise distances to a PHYLIP matrix
   `necom mat to-phylip input.tsv -o output.phy`
