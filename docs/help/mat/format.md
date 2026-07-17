Convert a PHYLIP matrix between formats and normalize it while preserving all distance values.

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
    * Names truncated to 10 characters.
    * Names left-aligned with space padding.
    * Distances in fixed-width format (6 decimal places).
    * Space-separated values.
    * Use this for compatibility with the original PHYLIP toolkit.

Examples:

1. Create a full matrix
   `necom mat format input.phy -o output.phy`

2. Create a lower-triangular matrix
   `necom mat format input.phy --format lower -o output.phy`

3. Create a strict PHYLIP matrix
   `necom mat format input.phy --format strict -o output.phy`
