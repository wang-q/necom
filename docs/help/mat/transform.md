Apply mathematical transformations to matrix elements.

Input:

* PHYLIP distance matrix or pairwise TSV file.
* Use `--input-format pair` to read pairwise TSV input.

Operations:

* `linear`: `val = val * scale + offset`.
* `inv-linear`: off-diagonal `val = max - val`; diagonal is set to `0`.
* `log`: `val = -ln(val)` (off-diagonal `<= 0` becomes `Inf`; diagonal `<= 0` becomes `0`).
* `exp`: `val = exp(-val)`.
* `square`: `val = val * val`.
* `sqrt`: `val = sqrt(val)` (negative values produce `NaN`).

Notes:

* Useful for converting similarity matrices to distance matrices.
* Use `--normalize` to normalize based on diagonal elements before transformation: `x_norm(i, j) = x(i, j) / sqrt(x(i, i) * x(j, j))`.
* When `--input-format pair` is used, `--same` and `--missing` control default diagonal and missing-pair values.
* Default parameter values: `--max-val 1.0`, `--scale 1.0`, `--offset 0.0`.

Examples:

1. Convert Identity (0-100) to Distance (0-1)
   `necom mat transform in.phy --op linear --scale -0.01 --offset 1.0`

2. Convert Identity (0-100) to Distance (0-100)
   `necom mat transform in.phy --op inv-linear --max-val 100`

3. Convert Similarity (0-1) to Distance (0-1)
   `necom mat transform in.phy --op inv-linear --max-val 1.0`

4. Log transformation with normalization
   `necom mat transform in.phy --op log --normalize`
