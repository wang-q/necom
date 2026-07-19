Calculate clustering evaluation metrics for partitions. Supports external comparison to a reference partition and internal evaluation using a distance matrix, tree, or coordinate matrix.

Input:

* A partition file (`p1`). Use `"stdin"` to read from standard input.
* `--other` / `--truth`, `--matrix`, `--tree`, `--coords` also accept `"stdin"`.

Output:

* TSV with a header row.
* External evaluation: one row of pair-based metrics (ARI, AMI, V-Measure, etc.).
* Internal evaluation: one row of distance-based or coordinate-based metrics (Silhouette, Dunn, ... or Davies-Bouldin, Calinski-Harabasz, ...).
* Batch mode (`--input-format long`): one row per `Group`, with the `Group` column preserved as the first column.

Notes:

* External Evaluation (Partition vs Partition): compares two partitions (e.g., ground truth vs result). Metrics include ARI, AMI, V-Measure, FMI, NMI, RI, Jaccard, Precision, and Recall.
    * The two partitions must cover exactly the same sample set. If one partition contains samples missing from the other, the command errors out instead of silently dropping them.
    * With `--no-singletons`, singleton clusters are first removed from `--other`; any samples that become unreferenced are excluded from evaluation, and the remaining sample sets must still match.
* Internal Evaluation (Partition + Matrix/Tree/Coords): evaluates a single partition without ground truth.
    * `--matrix` / `--tree`: distance-based metrics (Silhouette, Dunn, C-Index, Gamma, Tau). All samples in the partition must be present in the matrix or tree; otherwise the command errors out instead of producing `NaN` metrics.
    * `--coords`: coordinate-based metrics (Davies-Bouldin, Calinski-Harabasz, PBM, Ball-Hall, Xie-Beni, Wemmert-Gancarski). All samples in the partition must be present in the coordinate file.
* Empty partitions are rejected (single mode and each batch group must contain at least one sample).
* Batch Evaluation (Long Format): evaluates multiple partitions (e.g., from parameter scan). The input file must be in long format (`Group\tClusterID\tSampleID`).
* `--other` / `--truth`: second partition for external evaluation (synonyms).
* `--no-singletons`: exclude singleton clusters from `--other` before external evaluation.
* `--input-format`: supports `pair` (default), `cluster`, or `long` (required for batch mode).
* `--other-format`: format for the `--other` file (`cluster` or `pair`). Defaults to the value of `--input-format` in single mode, or `cluster` in batch mode (since the truth file is a single partition, not Long).
* In batch mode, the `Group` column is preserved as the first column of the output.
* Quick metric selection:
    * With ground truth: ARI or AMI.
    * Without ground truth (distance matrix or tree): Silhouette.
    * Without ground truth (coordinates): Davies-Bouldin or Calinski-Harabasz.
    * See [`docs/eval-partition.md`](../../eval-partition.md) for detailed metric definitions and the full selection guide.
* Typical batch workflow: generate candidates with `necom cut scan-simple`, evaluate with `--input-format long`, then select the best threshold.

Examples:

1. External evaluation: compare result with ground truth
   `necom eval partition result.tsv --other truth.tsv -o eval.tsv`

2. Internal evaluation using a distance matrix
   `necom eval partition result.tsv --matrix dist.phy`

3. Internal evaluation using a tree file
   `necom eval partition result.tsv --tree tree.nwk`

4. Internal evaluation using coordinate vectors
   `necom eval partition result.tsv --coords vectors.tsv`

5. Batch evaluation of scan results
   `necom eval partition partitions.tsv --input-format long --matrix dist.phy > scores.tsv`
