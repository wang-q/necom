Compare two PHYLIP distance matrices and calculate similarity metrics.

Input:

* Two PHYLIP distance matrices (full or lower-triangular).
* Only the intersection of common IDs is used for comparison.

Output:

* Tab-separated values with two columns: Method and Score.
* One row per requested method.

Methods:

* `all`: calculate all metrics below.
* `pearson`: Pearson correlation coefficient (-1 to 1).
* `spearman`: Spearman rank correlation (-1 to 1).
* `mae`: mean absolute error.
* `cosine`: cosine similarity (-1 to 1).
* `jaccard`: weighted Jaccard similarity (0 to 1).
* `euclid`: Euclidean distance.

Notes:

* Useful for evaluating consistency between distance calculation methods, or measuring information loss before and after clustering (Cophenetic Correlation).
* Default method is `pearson`.
* Multiple methods can be requested as a comma-separated list.

Examples:

1. Compare using Pearson correlation
   `necom mat compare matrix1.phy matrix2.phy --method pearson`

2. Compare using multiple methods
   `necom mat compare matrix1.phy matrix2.phy --method pearson,cosine,jaccard`
