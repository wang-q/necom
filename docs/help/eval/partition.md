Calculate clustering evaluation metrics for partitions. Supports external comparison to a reference partition and internal evaluation using a distance matrix, tree, or coordinate matrix.

Input:

* A partition file (`p1`). Use `"stdin"` to read from standard input.
* `--other` / `--truth`, `--matrix`, `--tree`, `--coords` also accept `"stdin"`.

Output:

* TSV with a header row.
* External evaluation: one row of pair-based metrics (`ari`, `ami`, `homogeneity`, `completeness`, `v_measure`, `fmi`, `nmi`, `mi`, `ri`, `jaccard`, `precision`, `recall`).
* Distance-based internal evaluation: `silhouette`, `dunn`, `c_index`, `gamma`, `tau`, `davies_bouldin`.
* Coordinate-based internal evaluation: `davies_bouldin`, `calinski_harabasz`, `pbm`, `ball_hall`, `xie_beni`, `wemmert_gancarski`.
* Batch mode (`--input-format long`): one row per `Group`, with `Group` as the first column.

Notes:

* Only one evaluation target may be provided per run: `--other` / `--truth`, `--matrix`, `--tree`, or `--coords`.
* Empty partitions are rejected in single mode and for each batch group.
* External evaluation requires the two partitions to cover exactly the same sample set.
* Internal evaluation requires every partition sample to be present in the distance source.
* With `--no-singletons`, singleton clusters are removed from `--other` before external evaluation.
* See [`docs/eval-partition.md`](../../eval-partition.md) for detailed metric definitions and selection guidance.

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
