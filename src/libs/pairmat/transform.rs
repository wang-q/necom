use super::named::NamedMatrix;

/// Apply a single transformation operation to a scalar value.
fn apply_transform(
    val: f32,
    method: &str,
    max_val: f32,
    scale: f32,
    offset: f32,
) -> anyhow::Result<f32> {
    Ok(match method {
        "linear" => val * scale + offset,
        "inv-linear" => max_val - val,
        "log" => {
            if val > 0.0 {
                -val.ln()
            } else {
                f32::INFINITY
            }
        }
        "exp" => (-val).exp(),
        "square" => val * val,
        "sqrt" => {
            if val >= 0.0 {
                val.sqrt()
            } else {
                f32::NAN
            }
        }
        _ => anyhow::bail!("unsupported transformation operation: {}", method),
    })
}

/// Apply mathematical transformations to a matrix element-wise.
///
/// Supports: linear, inv-linear, log, exp, square, sqrt.
/// When `normalize` is true, off-diagonal values are divided by `sqrt(d_i * d_j)`
/// and diagonal values are normalized to 1.0 (or 0.0 if the original diag <= 1e-9).
/// `sqrt` of a negative value returns `f32::NAN` to avoid hiding invalid input.
pub fn transform_matrix(
    matrix: &NamedMatrix,
    method: &str,
    max_val: f32,
    scale: f32,
    offset: f32,
    normalize: bool,
) -> anyhow::Result<NamedMatrix> {
    let mut result = matrix.clone();
    let size = result.size();

    // Get original diagonals (used for normalize and for transforming diagonal elements).
    // Missing diagonals are treated as zeros, matching the documentation for --normalize.
    let diags: Vec<f32> = result
        .get_diags()
        .cloned()
        .unwrap_or_else(|| vec![0.0; size]);
    let has_diags = result.get_diags().is_some();

    // Warn if normalize is requested but diagonals are missing or non-positive.
    // Non-positive diagonals trigger the `d_i <= 1e-9` branch in the normalize
    // step, which zeros out the corresponding off-diagonal values, making the
    // transformation a no-op for those rows/columns.
    if normalize {
        if !has_diags {
            log::warn!("--normalize requested but no diagonal values found; treating them as 0.0.");
        }
        // Start from NEG_INFINITY so all-negative diagonals report their true
        // max instead of being masked by a 0.0 initial value.
        let max_diag = diags.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        if max_diag <= 0.0 {
            log::warn!(
                "--normalize requested but all diagonal values are non-positive (max = {}).",
                max_diag
            );
        }
    }

    // Transform off-diagonal elements (upper triangle)
    for i in 0..size {
        for j in (i + 1)..size {
            let mut val = result.get(i, j);

            // 1. Normalize
            if normalize {
                let d_i = diags[i];
                let d_j = diags[j];
                if d_i > 1e-9 && d_j > 1e-9 {
                    val /= (d_i * d_j).sqrt();
                } else {
                    val = 0.0;
                }
            }

            // 2. Transform
            val = apply_transform(val, method, max_val, scale, offset)?;

            result.set(i, j, val);
        }
    }

    // Transform diagonal elements.
    // Normalize sets d to 1.0 (if original d > 1e-9) or 0.0, matching off-diagonal behavior
    // where x_norm(i,i) = x(i,i) / sqrt(x(i,i)*x(i,i)) = 1.0.
    let mut new_diags = vec![0.0; size];
    for i in 0..size {
        let mut d = if has_diags { diags[i] } else { 0.0 };
        if normalize {
            d = if d > 1e-9 { 1.0 } else { 0.0 };
        }
        d = match method {
            // Keep the diagonal at 0 for a valid distance matrix.
            "inv-linear" => 0.0,
            // log of a non-positive diagonal is defined as 0 (unlike off-diagonal Inf).
            "log" => {
                if d > 0.0 {
                    -d.ln()
                } else {
                    0.0
                }
            }
            _ => apply_transform(d, method, max_val, scale, offset)?,
        };
        new_diags[i] = d;
    }
    result.set_diags(new_diags)?;

    Ok(result)
}
