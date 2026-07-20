Apply mathematical transformations to matrix elements.

Input:

* PHYLIP distance matrix or pairwise TSV file.
* Use `--input-format pair` to read pairwise TSV input; useful for processing data from STDIN.

Notes:

* Available operations (`--op`):
    * `linear`: `val = val * scale + offset`.
    * `inv-linear`: off-diagonal `val = max - val`; diagonal is set to `0`.
    * `log`: `val = -ln(val)` (off-diagonal `<= 0` becomes `Inf`; diagonal `<= 0` becomes `0`).
    * `exp`: `val = exp(-val)` (input diagonals of `0` produce output diagonals of `1.0`).
    * `square`: `val = val * val`.
    * `sqrt`: `val = sqrt(val)` (negative values produce `NaN`).
* Useful for converting similarity matrices to distance matrices.
* Use `--normalize` to normalize based on diagonal elements before transformation: `x_norm(i, j) = x(i, j) / sqrt(x(i, i) * x(j, j))`.
    * `--normalize` requires diagonal data in the input; if absent, every diagonal is treated as `0.0`, so off-diagonal values become `0.0` before the selected `--op` is applied.
    * Diagonal values less than or equal to `1e-9` are treated as zero, producing `0.0` for the corresponding row/column.
    * Normalization is applied before the transformation op: off-diagonal `x(i,j) / sqrt(d_i*d_j)` then `--op`, diagonal normalized to `1.0` (or `0.0` if `d_i <= 1e-9`) then `--op`.
* When `--input-format pair` is used, `--same` and `--missing` control default diagonal and missing-pair values.
* Default parameter values: `--max-val 1.0`, `--scale 1.0`, `--offset 0.0`.

Examples:

1. Convert Identity (0-100) to Distance (0-1)
   `necom mat transform input.phy --op linear --scale -0.01 --offset 1.0 -o dist.phy`

2. Convert Identity (0-100) to Distance (0-100)
   `necom mat transform input.phy --op inv-linear --max-val 100 -o dist.phy`

3. Convert Similarity (0-1) to Distance (0-1)
   `necom mat transform input.phy --op inv-linear --max-val 1.0 -o dist.phy`

4. Probability to distance with log
   `necom mat transform input.phy --op log -o dist.phy`

5. Normalize raw scores then convert to distance
   `necom mat transform raw_scores.phy --normalize --op inv-linear --max-val 1.0 -o dist.phy`
