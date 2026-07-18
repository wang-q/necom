# necom File Formats

`necom` commands share a small set of file formats for clustering results, distance matrices, feature vectors, and phylogenetic trees. This document describes the common formats and points to command-specific details where applicable.

* For distance-matrix conversion and manipulation, see [`docs/mat.md`](mat.md).
* For Newick tree conventions (label quoting, branch-length handling), see [`docs/nwk.md`](nwk.md).
* For scan-mode partition output, see [`docs/cut.md`](cut.md#output-format-in-scan-mode).

## Partition Files

Used to represent clustering results (sample-to-cluster mapping). Three formats are supported via the `--format` option.

### Pair Format (`--format pair`)

The most general long-table format; each line is a `(representative, member)` pair.

* **Structure**: `Representative <tab> Member`
* **Representative selection**: For `dbscan` / `mcl` / `k-medoids`, controlled by `--rep {medoid|first}`; default `medoid`. `cc` does not read weights and always uses the alphabetically first member. The representative is written to the first column; the member to the second column. Singletons appear as `Name <tab> Name`.
* **Default format**: The default output format for flat clustering commands is `cluster`; use `--format pair` to emit this long-table representation.
* **Characteristics**: Easy to parse; supports streaming.
* **Example**:
  ```text
  GeneA	GeneA
  GeneA	GeneB
  GeneC	GeneC
  ```

### Cluster Format (`--format cluster`)

Wide-table format; each line represents a cluster containing all its members.

* **Structure**: tab-separated items, one cluster per line.
* **Characteristics**: Human-readable; suitable for inspecting results. The line number (1-based) is the ClusterID. The first item is the cluster representative when representative selection applies.
* **Example**:
  ```text
  GeneA	GeneB
  GeneC
  ```

### Long Format (batch, `--format long`)

A dedicated TSV format (`Group\tClusterID\tSampleID`) for batch evaluation, auto-emitted by `necom cut --scan` and consumed by `necom eval partition --input-format long`. See [`docs/cut.md`](cut.md#output-format-in-scan-mode) for the full specification.

## Distance Matrix

Used by `clust hier`, `nj`, `upgma`, `eval partition --matrix`, and `cut --dynamic-hybrid`.

### PHYLIP Format

`necom` accepts a relaxed PHYLIP format (arbitrary whitespace, optional header). See [`docs/mat.md`](mat.md#1-phylip-distance-matrix-dense) for full structure, strict vs relaxed variants, and lower-triangular form.

### Pairwise TSV

A sparse list representation of pairwise distances or similarities:

* **Format**: tab-separated three columns: `name1\tname2\tdistance`
* **Characteristics**: Suitable for sparse graphs or as an exchange format with other tools (e.g., BLAST/MMseqs2).
* **Conversion**: Use `necom mat to-phylip` to assemble into a PHYLIP matrix, and `necom mat to-pair` to flatten a PHYLIP matrix into this form. See [`docs/mat.md`](mat.md#2-pairwise-tsv-sparse-list-form) for details.

## Coordinates / Feature Vectors

Used by `eval partition --coords` (Davies-Bouldin Index) or future `kmeans/gmm`.

### FeatureVector Format

* **Structure**: `Name <tab> Val1,Val2,Val3...`
* **Delimiters**: **Tab** between name and vector; **commas** between numeric values.
* **Example**:
  ```text
  GeneA	1.2,0.5,3.3
  GeneB	1.1,0.6,3.1
  ```
* **Compatibility**: A general feature-vector/coordinate representation format.

## Newick Tree Conventions

`necom` uses the Newick format for phylogenetic and hierarchical-clustering trees. Important conventions include label quoting for reserved characters and normalization of non-finite branch lengths. See [`docs/nwk.md`](nwk.md) for the full specification.