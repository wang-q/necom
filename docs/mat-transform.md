# necom mat transform

The `necom mat transform` command applies mathematical transformations to values in a matrix.

It is the core tool for converting a **Similarity Matrix** into a **Distance Matrix**, and also supports normalization and other numerical adjustments.

> This document is an in-depth explanation of the `mat transform` subcommand. For an overview of all `mat` subcommands, see [mat.md](mat.md).

## Usage

```bash
necom mat transform [OPTIONS] <infile>
```

### Arguments

- `<infile>`: Input PHYLIP matrix or Pairwise TSV file.

### Options

- `--input-format <FORMAT>`: Input format (default: `phylip`, optional: `pair`).
  - Explicitly specifying `--input-format pair` is useful for processing TSV data from pipe (STDIN) input.
- `--op <METHOD>`: Transformation operation (default: `linear`).
  - `linear`: $val = val \times scale + offset$
  - `inv-linear`: $val = max - val$
  - `log`: $val = -\ln(val)$ (values $\le 0$ are set to 0 or Inf)
  - `exp`: $val = \exp(-val)$
  - `square`: $val = val^2$
  - `sqrt`: $val = \sqrt{val}$
- `--max-val <FLOAT>`: Maximum value used for `inv-linear` (default: 1.0).
- `--scale <FLOAT>`: Scale factor used for `linear` (default: 1.0).
- `--offset <FLOAT>`: Offset used for `linear` (default: 0.0).
- `--normalize`: Whether to normalize based on diagonal elements before transformation (requires diagonal data in the matrix).
  - Normalization formula: $x_{norm}(i, j) = \frac{x(i, j)}{\sqrt{x(i, i) \times x(j, j)}}$
  - **Why normalize?**
    - Raw scores are usually affected by sequence length and cannot be directly compared (e.g., a score of 1000 for a long sequence may be less significant than a score of 100 for a short sequence).
    - Normalization uses the diagonal (self-alignment score) to convert raw scores into relative similarity (0–1 range), giving subsequent distance transformations (e.g., $1-S$) a meaningful mathematical interpretation.
- `-o, --outfile <outfile>`: Output filename (default: stdout).

## Common Scenarios

### 1. Identity (0–100) to Distance (0–1)

Identity values output by tools such as BLAST are usually in the range 0 to 100.
Target formula: $D = (100 - Identity) / 100 = 1 - 0.01 \times Identity$.

Using the `linear` operation:
```bash
necom mat transform input.phy --op linear --scale -0.01 --offset 1.0 -o dist.phy
```

Or in two steps (invert first, then scale):
```bash
necom mat transform input.phy --op inv-linear --max-val 100 | \
necom mat transform stdin --op linear --scale 0.01 -o dist.phy
```

### 2. Identity (0–100) to Distance (0–100)

Simple inversion: $D = 100 - Identity$.

```bash
necom mat transform input.phy --op inv-linear --max-val 100 -o dist.phy
```

### 3. Similarity (0–1) to Distance (0–1)

Standard linear inversion: $D = 1.0 - S$.

```bash
necom mat transform input.phy --op inv-linear --max-val 1.0 -o dist.phy
```

### 4. Probability / Multiplicative Model Conversion (Log)

Convert sequence identity probability into evolutionary distance (similar to the first step of Jukes-Cantor correction).
$D = -\ln(S)$.

```bash
# Assume the input matrix is a probability matrix with diagonal 1.0
necom mat transform input.phy --op log -o dist.phy
```

### 5. Normalize and Transform

If the input is unnormalized similarity scores (e.g., Alignment Score) and the matrix contains diagonal (self-alignment) values:
normalize to 0–1 first, then convert to distance.

```bash
# 1. Normalize: S_norm = S_ij / sqrt(S_ii * S_jj)
# 2. Transform: D = 1.0 - S_norm
necom mat transform raw_scores.phy --normalize --op inv-linear --max-val 1.0 -o dist.phy
```

## Background and Principles

Clustering algorithms (such as UPGMA, NJ, Ward) and multidimensional scaling (MDS) usually require a **Distance Matrix** or **Dissimilarity Matrix** that satisfies:

- $D(x, x) = 0$
- $D(x, y) \ge 0$
- Smaller $D(x, y)$ indicates higher similarity

However, upstream bioinformatics tools (such as BLAST, MMseqs2, Diamond) or statistical analyses usually output **Similarity**, which satisfies:

- $S(x, x) = Max$ (e.g., 1.0 or 100)
- Larger $S(x, y)$ indicates higher similarity

Users currently need to use `awk` or external scripts for conversion (e.g., `100 - identity`), which is inconvenient and error-prone (e.g., missing values or self-alignments may not be handled).

### Conversion Models

`necom mat transform` supports the following common transformation modes for converting similarity to distance or performing other mathematical processing:

#### 1. Linear Inversion

Applicable to similarities with a fixed upper bound (e.g., Identity, Percent Similarity).
$$D = Max - S$$
- **Scenario**: BLAST Identity (0–100) $\rightarrow$ $D = 100 - S$
- **Scenario**: Fraction (0–1) $\rightarrow$ $D = 1 - S$

#### 2. Normalized Linear Inversion

If $S$ has no fixed upper bound (e.g., Alignment Score), normalization is required first.
$$D = 1 - \frac{S(x, y)}{\sqrt{S(x, x) \cdot S(y, y)}}$$
Or simply:
$$D = 1 - \frac{S(x, y)}{Max(S)}$$

#### 3. Logarithmic

Applicable to probabilities or multiplicative models (similar to Jukes-Cantor correction).
$$D = -\ln(S)$$
Or after normalization:
$$D = -\ln(\frac{S(x, y)}{\sqrt{S(x, x) \cdot S(y, y)}})$$
- **Scenario**: Sequence identity probability $\rightarrow$ evolutionary distance

#### 4. Reciprocal [Not Implemented]
$$D = \frac{1}{S} - \frac{1}{Max}$$
- **Scenario**: Rarely used, for converting certain physical quantities.
- **Status**: Not currently implemented as an op in `necom mat transform`; can be approximated with `--op linear` plus an external script.

#### 5. Special Transformations [Not Implemented]
- **Cosine Similarity**: $D = 1 - \cos(\theta)$
- **Correlation**: $D = \sqrt{2(1 - r)}$ or $D = 1 - r$
- **Status**: Not currently implemented in `necom mat transform`. For Cosine/Correlation distances, we recommend computing them in Python (SciPy) and exporting the result as a PHYLIP matrix.

## Notes

- **Diagonal handling**:
  - When `necom` reads a matrix, it usually ignores the diagonal (sets it to 0), but the `transform` command attempts to preserve diagonal information to support `--normalize`.
  - If the input file lacks diagonal information (as in some PHYLIP variants), `--normalize` will not work correctly (treated as 0).
- **Numerical stability**:
  - The `log` operation is sensitive to 0 or negative values; the program handles them as very large values or 0.
  - If the diagonal is 0 during normalization, the result will be 0.
