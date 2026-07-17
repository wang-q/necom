# necom mat

The `necom mat` module focuses on the manipulation and conversion of **distance matrices**. It is the upstream data preparation and preprocessing toolkit for `necom clust` (clustering and tree inference).

## Core Purpose

- **Input/Output**: Primarily handles **PHYLIP** format distance matrices (dense) and **Pairwise TSV** (sparse/list) format.
- **Functionality**: Format conversion, subset extraction, matrix comparison, and standardization.
- **Goal**: Provide standard, efficient data interfaces for phylogenetics and statistical clustering.

## Supported File Formats

`necom` currently focuses on clustering, distance matrix processing, and phylogenetic tree operations. Supported file formats are limited to those actually used by these commands.

| Format | Description | Command Docs |
| :--- | :--- | :--- |
| Distance | Distance matrix structures such as `PHYLIP` and `Pairwise` | [mat.md](mat.md), [clust.md](clust.md) |
| Newick | Phylogenetic tree format | [nwk.md](nwk.md) |

Distance matrices and Newick trees do not carry genomic coordinates. If other `necom` commands require coordinate-based input, this is documented separately in the corresponding command documentation.

## Supported Formats and Data Structures

`necom` supports two external distance matrix file formats: `PHYLIP` and `Pairwise`. (Internal matrix data structures are documented in [`notes/design/mat-impl.md`](../notes/design/mat-impl.md).)

### 1. PHYLIP Distance Matrix (Dense)

The `PHYLIP` distance matrix format is a common format in phylogenetic analysis. `necom` provides a series of tools for processing this format.

`necom` stores this internally using the `NamedMatrix` structure, backed by `CondensedMatrix` (a one-dimensional array storing the upper or lower triangle), with memory usage of approximately $O(N^2/2)$.

`necom` supports both Strict and Relaxed `PHYLIP` formats.

**Relaxed PHYLIP (default input support)**:
- First line: number of samples (usually required; if omitted, the program attempts to infer it automatically from the data rows).
- Data rows: sample name followed by distance values.
- Separators: whitespace characters (spaces or tabs).
- Matrix form: supports full square matrix or lower triangular matrix. The lower-triangular form may either include the diagonal (row `i` has `i+1` values) or omit it (row `i` has `i` values), in which case the diagonal is assumed to be `0.0`.
- Name length: not restricted.
- Note: if the first data row starts with a numeric token (e.g. `123`), it may be mistaken for the optional sequence-count header. Use non-numeric prefixes or omit the header line to avoid ambiguity.

**Strict PHYLIP (`strict` mode output)**:
- Follows the original `PHYLIP` standard.
- Sequence names: strictly truncated to 10 characters, left-aligned and space-padded.
- Numeric format: space-separated, usually kept to 6 decimal places.

Variants:
- **Full**: Standard $N \times N$ matrix including the redundant symmetric portion.
- **Lower-triangular**: Only the lower-triangle portion, reducing file size by half.

### 2. Pairwise TSV (Sparse/List Form)

The `Pairwise` format is a simple three-column TSV format for representing pairwise distances between sequences, commonly used as an intermediate format or input for graph data.

Sparse or list-form distance data, suitable for storing graph structures or only a subset of pairs.

- **Format**: tab-separated three columns: `name1\tname2\tdistance`
- **Characteristics**:
  - Suitable for sparse graphs or as an exchange format with other tools (e.g., BLAST/MMseqs2).
  - When converting to a matrix, unlisted pairs are treated as missing values or defaults.

`necom` provides mutual conversion between matrix and `Pairwise` list:
- **Matrix to Pair (`necom mat to-pair`)**: flatten a `PHYLIP` matrix into a `Pairwise` list.
- **Pair to Matrix (`necom mat to-phylip`)**: assemble a `Pairwise` list back into a `PHYLIP` matrix, supporting `--missing` and `--same` parameters.

## Subcommands in Detail

### Format Conversion

#### `necom mat to-phylip`

Build a full PHYLIP distance matrix from a pairwise TSV (e.g., alignment output). Missing pairs can be filled with `--missing` and self-pairs with `--same`.

#### `necom mat to-pair`

Flatten a PHYLIP matrix into a three-column pairwise TSV (`A B distance`), emitting the lower triangle including the diagonal.

#### `necom mat format`

Convert a PHYLIP matrix into another PHYLIP variant while preserving all distance values.

* `--format full` (default): full square matrix with original names.
* `--format lower`: lower-triangular matrix without diagonal values.
* `--format strict`: 10-character names and fixed-width values for compatibility with the original PHYLIP toolkit.

### Operations and Analysis

#### `necom mat subset`

Extract a PHYLIP submatrix in the order of a given ID list.

#### `necom mat compare`

Compare two matrices on their common IDs using correlation, error, or distance metrics. Multiple `--method` values can be comma-separated.

#### `necom mat transform`

Apply mathematical transformations to matrix elements. It is the main tool for converting a **Similarity Matrix** into a **Distance Matrix**, and also supports normalization and other numerical adjustments.

Clustering algorithms (UPGMA, NJ, Ward) and multidimensional scaling require a **Distance Matrix** with $D(x, x) = 0$, $D(x, y) \ge 0$, and smaller values indicating higher similarity. Upstream tools such as BLAST, MMseqs2, and Diamond usually output **Similarity**, where $S(x, x) = Max$ and larger values indicate higher similarity.

**Key options**:

* `--op`: `linear`, `inv-linear`, `log`, `exp`, `square`, `sqrt`.
* `--normalize`: scale values by diagonal elements before transformation.
* `--input-format pair`: read pairwise TSV instead of PHYLIP.

#### Conversion Models

`necom mat transform` supports common similarity-to-distance conversions:

##### 1. Linear Inversion

For similarities with a fixed upper bound:

$$D = Max - S$$

Examples: BLAST identity (0–100) becomes $D = 100 - S$; fractions become $D = 1 - S$.

##### 2. Normalized Linear Inversion

For raw scores without a fixed upper bound, normalize by diagonal self-scores first:

$$D = 1 - \frac{S(x, y)}{\sqrt{S(x, x) \cdot S(y, y)}}$$

A simpler alternative uses the global maximum: $D = 1 - S(x, y) / Max(S)$.

##### 3. Logarithmic

For probabilities or multiplicative models:

$$D = -\ln(S)$$

After normalization: $D = -\ln(S(x, y) / \sqrt{S(x, x) \cdot S(y, y)})$. Useful for converting sequence identity probability to evolutionary distance.

#### Notes

* Diagonal information is preserved so that `--normalize` can use self-scores. If the input lacks diagonal values, `--normalize` will not work correctly.
* `log` of 0 or negative off-diagonal values produces `Inf`; non-positive diagonal values become `0`.

#### Future Work

The following conversions are not yet implemented:

* **Reciprocal**: $D = 1/S - 1/Max$ (can be approximated with `--op linear`).
* **Cosine Similarity**: $D = 1 - \cos(\theta)$.
* **Correlation**: $D = \sqrt{2(1 - r)}$ or $D = 1 - r$.

For Cosine/Correlation distances, compute them in Python (SciPy) and export as a PHYLIP matrix.

## Recommended Workflows

### Scenario A: Tree Inference from BLAST Results

```bash
# 1. Parse BLAST results into pairwise distances (assuming distance = 1 - identity has already been computed)
# Note: ensure both A-B and B-A are present, or rely on a single direction
awk '{print $1, $2, 100-$3}' blast.out > pairs.tsv

# 2. Convert to PHYLIP matrix; set unaligned pairs to maximum distance 100
necom mat to-phylip pairs.tsv --missing 100 -o matrix.phy

# 3. Build NJ tree
necom clust nj matrix.phy > tree.nwk
```

### Scenario B: Extract Subset for Fine-Grained Analysis

```bash
# 1. Prepare a list of IDs of interest
cat interesting_ids.txt
# gene_A
# gene_B
# ...

# 2. Extract submatrix from a whole-genome distance matrix
necom mat subset genome_dist.phy interesting_ids.txt -o sub_matrix.phy

# 3. Analyze the subset with Ward clustering
necom clust hier sub_matrix.phy --method ward > sub_tree.nwk
```

### Scenario C: Evaluate Consistency Between Two Distance Calculation Methods

```bash
# Compare distance matrices based on K-mer (mash) and alignment (ani)
necom mat compare mash_dist.phy ani_dist.phy --method pearson,spearman

# Example output:
# Sequences in matrices: 100 and 100
# Common sequences: 100
# Method    Score
# pearson   0.985432
# spearman  0.971234
```

### Scenario D: Prepare Data for the Phylip Software Package

```bash
# Convert a long-name matrix to strict Phylip format
necom mat format modern.phy --format strict -o input.infile

# Then run neighbor (the original Phylip program)
neighbor < input.infile
```
