use std::collections::HashSet;
use std::io::Write;

use super::NamedMatrix;

/// PHYLIP matrix output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixFormat {
    Full,
    Lower,
    Strict,
}

impl MatrixFormat {
    /// Parse a format name (`"full"` / `"lower"` / `"strict"`) into a `MatrixFormat`.
    pub fn from_mode(s: &str) -> anyhow::Result<Self> {
        match s {
            "full" => Ok(Self::Full),
            "lower" => Ok(Self::Lower),
            "strict" => Ok(Self::Strict),
            _ => anyhow::bail!("unsupported output format: {}", s),
        }
    }
}

/// Write a NamedMatrix in the specified PHYLIP format.
///
/// `precision` controls the number of decimal places for `Full` and `Lower`
/// formats. `None` prints raw values; `Some(n)` prints `n` decimal places.
/// `Strict` format always uses 6 decimal places as required by the PHYLIP standard.
pub fn write_phylip_matrix<W: Write>(
    m: &NamedMatrix,
    fmt: MatrixFormat,
    precision: Option<usize>,
    writer: &mut W,
) -> anyhow::Result<()> {
    let names = m.get_names();
    let size = m.size();

    writeln!(writer, "{}", size)?;

    for (i, name) in names.iter().enumerate().take(size) {
        match fmt {
            MatrixFormat::Full => {
                write!(writer, "{}", name)?;
                for j in 0..size {
                    writer.write_all(b"\t")?;
                    match precision {
                        Some(p) => write!(writer, "{:.1$}", m.get(i, j), p)?,
                        None => write!(writer, "{}", m.get(i, j))?,
                    }
                }
            }
            MatrixFormat::Lower => {
                write!(writer, "{}", name)?;
                for j in 0..i {
                    writer.write_all(b"\t")?;
                    match precision {
                        Some(p) => write!(writer, "{:.1$}", m.get(i, j), p)?,
                        None => write!(writer, "{}", m.get(i, j))?,
                    }
                }
            }
            MatrixFormat::Strict => {
                // Strict PHYLIP reserves exactly 10 bytes for the name.
                let truncated = if name.len() <= 10 {
                    name.as_str()
                } else {
                    let mut end = 10;
                    while end > 0 && !name.is_char_boundary(end) {
                        end -= 1;
                    }
                    &name[..end]
                };
                writer.write_all(truncated.as_bytes())?;
                for _ in truncated.len()..10 {
                    writer.write_all(b" ")?;
                }
                for j in 0..size {
                    write!(writer, " {:.6}", m.get(i, j))?;
                }
            }
        }
        writeln!(writer)?;
    }

    Ok(())
}

/// Write a submatrix restricted to `names`. Returns the list of names not found in `m`.
///
/// `precision` controls decimal places; `None` prints raw values.
pub fn write_subset<W: Write>(
    m: &NamedMatrix,
    names: &[String],
    precision: Option<usize>,
    writer: &mut W,
) -> anyhow::Result<Vec<String>> {
    let all_names = m.get_names();
    let mut indices = Vec::new();
    let mut missing = Vec::new();

    for name in names {
        match m.get_index(name) {
            Some(idx) => indices.push(idx),
            None => missing.push(name.clone()),
        }
    }

    writeln!(writer, "{}", indices.len())?;

    let write_value = |writer: &mut W, value: f32| -> anyhow::Result<()> {
        match precision {
            Some(p) => write!(writer, "{:.1$}", value, p)?,
            None => write!(writer, "{}", value)?,
        }
        Ok(())
    };

    for &i in &indices {
        write!(writer, "{}", all_names[i])?;
        for &j in &indices {
            writer.write_all(b"\t")?;
            write_value(writer, m.get(i, j))?;
        }
        writeln!(writer)?;
    }

    Ok(missing)
}

/// Extract paired values from the lower triangle (excluding diagonal) of two matrices,
/// restricted to sequence names common to both. Returns `(common_names, values1, values2)`.
///
/// Builds an index map once (O(N)) so the nested pairwise loop uses direct `get(i, j)`
/// indexing instead of repeated `get_by_name` hash lookups (O(N^2) lookups avoided).
pub fn extract_common_lower_triangle(
    m1: &NamedMatrix,
    m2: &NamedMatrix,
) -> anyhow::Result<(Vec<String>, Vec<f32>, Vec<f32>)> {
    let names1 = m1.get_names();
    let names2 = m2.get_names();
    let names2_set: HashSet<&str> = names2.iter().map(|s| s.as_str()).collect();
    let common_names: Vec<String> = names1
        .iter()
        .filter(|name| names2_set.contains(name.as_str()))
        .map(|s| s.to_string())
        .collect();

    if common_names.len() < 2 {
        anyhow::bail!(
            "at least 2 common sequence names required for comparison, found {}",
            common_names.len()
        );
    }

    // Pre-compute (idx_in_m1, idx_in_m2) for each common name so the O(N^2)
    // pairwise loop uses direct indexing instead of hash lookups.
    let indices: Vec<(usize, usize)> = common_names
        .iter()
        .map(|name| {
            // Safety: names come from the intersection of both matrices, so both
            // lookups must succeed.
            let i1 = m1.get_index(name).expect("common name missing in m1");
            let i2 = m2.get_index(name).expect("common name missing in m2");
            (i1, i2)
        })
        .collect();

    let n = common_names.len();
    let mut values1 = Vec::with_capacity(n * (n - 1) / 2);
    let mut values2 = Vec::with_capacity(n * (n - 1) / 2);

    for (i, &(i1, i2)) in indices.iter().enumerate() {
        for &(j1, j2) in indices.iter().take(i) {
            values1.push(m1.get(i1, j1));
            values2.push(m2.get(i2, j2));
        }
    }

    Ok((common_names, values1, values2))
}
