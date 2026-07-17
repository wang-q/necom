Calculate clustering evaluation metrics for partitions. Supports external comparison to a reference partition and internal evaluation using a distance matrix, tree, or coordinate matrix.

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
* Quick metric selection:
    * With ground truth: ARI or AMI.
    * Without ground truth (distance matrix or tree): Silhouette.
    * Without ground truth (coordinates): Davies-Bouldin or Calinski-Harabasz.
    * See `docs/clust-eval.md` for detailed metric definitions and the full selection guide.
* Typical batch workflow: generate candidates with `necom cut --scan`, evaluate with `--input-format long`, then select the best threshold.

Examples:

1. External evaluation: compare result with ground truth
   `necom clust eval result.tsv --other truth.tsv -o eval.tsv`

2. Internal evaluation using a distance matrix
   `necom clust eval result.tsv --matrix dist.phy`

3. Internal evaluation using a tree file
   `necom clust eval result.tsv --tree tree.nwk`

4. Internal evaluation using coordinate vectors
   `necom clust eval result.tsv --coords vectors.tsv`

5. Batch evaluation of scan results
   `necom clust eval partitions.tsv --input-format long --matrix dist.phy > scores.tsv`
