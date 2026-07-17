Extract a submatrix from a PHYLIP matrix using a list of names.

Input:

* Matrix: PHYLIP distance matrix (full or lower-triangular).
* List: one name per line.
* Empty lines and lines starting with `#` in the list file are ignored.

Output:

* A full PHYLIP distance matrix containing only the requested names.
* Rows and columns follow the order given in the list file.

Notes:

* Useful for extracting specific species or gene families from a large matrix.
* Can also be used to reorder a matrix by providing names in the desired order.
* Names in the list that are not found in the matrix are reported as warnings and skipped.

Examples:

1. Create a full submatrix
   `necom mat subset input.phy list.txt -o output.phy`
