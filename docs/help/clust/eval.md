Calculate clustering evaluation metrics.

Input:

* A partition file (`p1`).

Modes:

1. External Evaluation (Partition vs Partition): compares two partitions (e.g., ground truth vs result). Metrics include ARI, AMI, V-Measure, FMI, NMI, RI, Jaccard, Precision, and Recall.
2. Internal Evaluation (Partition + Matrix/Tree/Coords): evaluates a single partition without ground truth.
    * `--matrix` / `--tree`: distance-based metrics (Silhouette, Dunn, C-Index, Gamma, Tau).
    * `--coords`: coordinate-based metrics (Davies-Bouldin, Calinski-Harabasz, PBM, Ball-Hall, Xie-Beni, Wemmert-Gancarski).
3. Batch Evaluation (Long Format): evaluates multiple partitions (e.g., from parameter scan). The input file must be in long format (`Group\tCluster\tSample`).

Notes:

* `--other` / `--truth`: second partition for external evaluation (synonyms).
* `--no-singletons`: exclude singleton clusters from `--other` before external evaluation.
* `--input-format`: supports `pair` (default), `cluster`, or `long` (required for batch mode).
* In batch mode, the `Group` column is preserved as the first column of the output.

Examples:

1. Compare result with ground truth
   `necom clust eval result.tsv --other truth.tsv -o eval.tsv`

2. Evaluate result using a distance matrix
   `necom clust eval result.tsv --matrix dist.phy`

3. Batch evaluation of scan results
   `necom clust eval partitions.tsv --format long --matrix dist.phy > scores.tsv`
