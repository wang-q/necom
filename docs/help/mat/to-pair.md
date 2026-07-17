Convert a PHYLIP distance matrix to pairwise distances.

Input:

* PHYLIP distance matrix (full or lower-triangular).
* First line can be sequence count (optional).
* Each line: sequence name followed by distances.

Output:

* Tab-separated values (TSV).
* Three columns: name1, name2, distance.
* Lower-triangular output, including the diagonal.
* Useful as an edge list for graph clustering (e.g., `mcl`) or network visualization (e.g., Cytoscape).

Examples:

1. Convert a PHYLIP matrix
   `necom mat to-pair input.mat -o output.tsv`

2. Output to screen
   `necom mat to-pair input.mat`
