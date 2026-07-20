Convert a PHYLIP matrix between formats.

Input:

* PHYLIP distance matrix (full or lower-triangular).
* Optional first line: number of sequences.
* Each line: sequence name followed by distances.

Output:

* `full` (default):
    * Full square matrix.
    * Tab-separated values.
    * Original sequence names preserved.
* `lower`:
    * Lower triangular matrix without diagonal values.
    * Tab-separated values.
    * Original sequence names preserved.
    * Row i contains i tab-separated values.
    * Useful for saving disk space.
* `strict`:
    * Standard PHYLIP format.
    * Names truncated to 10 bytes.
    * Names left-aligned with space padding.
    * Distances formatted to 6 decimal places, space-separated.
    * Space-separated values.
    * Use this for compatibility with the original PHYLIP toolkit.

Notes:

* `strict` mode truncates names to 10 bytes and formats distances to 6 decimal places, which can cause name collisions and precision loss.
* `full` and `lower` preserve original names without truncation.
* `full` and `lower` preserve distance values as-is; `strict` may round them to 6 decimal places.

Examples:

1. Create a full matrix
   `necom mat format input.phy -o output.phy`

2. Create a lower-triangular matrix
   `necom mat format input.phy --format lower -o output.phy`

3. Create a strict PHYLIP matrix
   `necom mat format input.phy --format strict -o output.phy`
