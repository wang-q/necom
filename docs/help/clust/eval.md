Calculate clustering evaluation metrics.

Input:

* A partition file.

Modes:

1. External Evaluation (Partition vs Partition): compares two partitions (e.g., ground truth vs result). Metrics include ARI, AMI, and V-Measure.
2. Internal Evaluation (Partition + Matrix): evaluates a single partition using a distance matrix. Metrics include Silhouette Coefficient.
3. Batch Evaluation (Long Format): evaluates multiple partitions (e.g., from parameter scan) against a ground truth or using internal metrics. The input file must be in long format (`Group\tCluster\tSample`).

Examples:

1. Compare result with ground truth
   `necom clust eval result.tsv --other other.tsv -o eval.tsv`

2. Evaluate result using a distance matrix
   `necom clust eval result.tsv --matrix dist.phy`

3. Batch evaluation of scan results
   `necom clust eval scan.tsv --format long --matrix dist.phy`
