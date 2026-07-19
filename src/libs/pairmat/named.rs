use std::collections::HashSet;
use std::io::BufRead;

use indexmap::IndexMap;

use super::condensed::{get_condensed_index, CondensedMatrix};
use super::scoring::ScoringMatrix;

/// Build a name-to-index map from a list of unique sequence names.
fn build_name_map(names: Vec<String>) -> anyhow::Result<IndexMap<String, usize>> {
    let size = names.len();
    let mut map = IndexMap::with_capacity(size);
    for (i, name) in names.into_iter().enumerate() {
        if map.insert(name.clone(), i).is_some() {
            anyhow::bail!("duplicate sequence name: {}", name);
        }
    }
    Ok(map)
}

/// A named matrix for storing pairwise distances/scores with sequence names.
///
/// Wraps a `CondensedMatrix` internally to save memory (N(N-1)/2).
/// Assumes symmetric matrix with 0 diagonal (distance matrix).
#[derive(Debug, Clone)]
pub struct NamedMatrix {
    names: indexmap::IndexMap<String, usize>,
    matrix: CondensedMatrix,
    diags: Option<Vec<f32>>,
}

impl NamedMatrix {
    /// Create a new named matrix from a list of unique sequence names.
    pub fn new(names: Vec<String>) -> anyhow::Result<Self> {
        let names_map = build_name_map(names)?;
        let matrix = CondensedMatrix::new(names_map.len());

        Ok(NamedMatrix {
            names: names_map,
            matrix,
            diags: None,
        })
    }

    /// Create from existing names and values (condensed upper triangle).
    pub fn new_from_values(
        names: Vec<String>,
        values: Vec<f32>,
    ) -> anyhow::Result<Self> {
        let names_map = build_name_map(names)?;
        let matrix = CondensedMatrix::from_vec(names_map.len(), values)?;

        Ok(NamedMatrix {
            names: names_map,
            matrix,
            diags: None,
        })
    }

    /// Create with numeric names ("0", "1", ...).
    pub fn with_ids(size: usize) -> Self {
        let matrix = CondensedMatrix::new(size);
        let mut names_map = IndexMap::with_capacity(size);
        for i in 0..size {
            names_map.insert(i.to_string(), i);
        }

        NamedMatrix {
            names: names_map,
            matrix,
            diags: None,
        }
    }

    /// Number of rows/columns in the matrix.
    pub fn size(&self) -> usize {
        self.matrix.size()
    }

    /// Consume the NamedMatrix and return its parts (names, condensed matrix).
    pub fn into_parts(self) -> (Vec<String>, CondensedMatrix) {
        let names = self.names.into_keys().collect();
        (names, self.matrix)
    }

    /// Access the underlying CondensedMatrix
    pub fn matrix(&self) -> &CondensedMatrix {
        &self.matrix
    }

    /// Get value at `(row, col)`. Returns the stored diagonal when `row == col`.
    pub fn get(&self, row: usize, col: usize) -> f32 {
        if row == col {
            if let Some(ref diags) = self.diags {
                return diags[row];
            }
        }
        self.matrix.get(row, col)
    }

    /// Set value at (row, col).
    ///
    /// Diagonal values are only stored if `set_diags` has been called;
    /// otherwise `set(i, i, _)` is silently ignored and `get(i, i)` returns 0.0.
    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        if row == col {
            if let Some(ref mut diags) = self.diags {
                diags[row] = value;
            }
        } else {
            self.matrix.set(row, col, value)
        }
    }

    /// Linear condensed index for `(row, col)`. Requires `row != col`.
    pub fn index(&self, row: usize, col: usize) -> usize {
        let (r, c) = if row < col { (row, col) } else { (col, row) };
        get_condensed_index(self.size(), r, c)
    }

    /// Return all names in insertion order.
    pub fn get_names(&self) -> Vec<&String> {
        self.names.keys().collect()
    }

    /// Return the row/column index for `name`, if present.
    pub fn get_index(&self, name: &str) -> Option<usize> {
        self.names.get(name).copied()
    }

    /// Replace the diagonal vector. Length must equal `size()`.
    pub fn set_diags(&mut self, diags: Vec<f32>) -> anyhow::Result<()> {
        if diags.len() == self.size() {
            self.diags = Some(diags);
            Ok(())
        } else {
            anyhow::bail!(
                "diagonal length {} does not match matrix size {}",
                diags.len(),
                self.size()
            )
        }
    }

    /// Borrow the diagonal vector, if set.
    pub fn get_diags(&self) -> Option<&Vec<f32>> {
        self.diags.as_ref()
    }

    /// Get the underlying condensed data vector.
    pub fn values(&self) -> &[f32] {
        self.matrix.data()
    }

    /// Get matrix value by sequence names
    ///
    /// ```
    /// # use necom::libs::pairmat::NamedMatrix;
    /// let names = vec!["seq1".to_string(), "seq2".to_string()];
    /// let mut matrix = NamedMatrix::new(names).unwrap();
    /// matrix.set(0, 1, 0.5);
    ///
    /// assert_eq!(matrix.get_by_name("seq1", "seq2"), Some(0.5));
    /// assert_eq!(matrix.get_by_name("seq1", "seq3"), None);  // Non-existent name
    /// ```
    pub fn get_by_name(&self, name1: &str, name2: &str) -> Option<f32> {
        let i = self.names.get(name1)?;
        let j = self.names.get(name2)?;
        Some(self.get(*i, *j))
    }

    /// Set matrix value by sequence names
    ///
    /// ```
    /// # use necom::libs::pairmat::NamedMatrix;
    /// let names = vec!["seq1".to_string(), "seq2".to_string()];
    /// let mut matrix = NamedMatrix::new(names).unwrap();
    ///
    /// assert!(matrix.set_by_name("seq1", "seq2", 0.5).is_ok());
    /// assert_eq!(matrix.get_by_name("seq1", "seq2"), Some(0.5));
    /// assert!(matrix.set_by_name("seq1", "seq3", 0.5).is_err());  // Non-existent name
    /// ```
    pub fn set_by_name(
        &mut self,
        name1: &str,
        name2: &str,
        value: f32,
    ) -> anyhow::Result<()> {
        match (self.names.get(name1), self.names.get(name2)) {
            (Some(&i), Some(&j)) => {
                self.set(i, j, value);
                Ok(())
            }
            (None, _) => anyhow::bail!("Name not found: {}", name1),
            (_, None) => anyhow::bail!("Name not found: {}", name2),
        }
    }

    /// Build a NamedMatrix from a 3-column pairwise TSV (`name1<tab>name2<tab>score`).
    /// Self-pairs default to `same`; missing pairs default to `missing`.
    pub fn from_pair_scores(
        infile: &str,
        same: f32,
        missing: f32,
    ) -> anyhow::Result<Self> {
        let (scoring_matrix, index_name) =
            ScoringMatrix::from_pair_scores(infile, same, missing)?;
        let size = index_name.len();

        // Create NamedMatrix from ScoringMatrix
        let mut matrix = NamedMatrix::new(index_name.into_iter().collect())?;
        let mut diags = vec![same; size];

        for (i, d) in diags.iter_mut().enumerate() {
            *d = scoring_matrix.get(i, i);
            for j in (i + 1)..size {
                matrix.set(i, j, scoring_matrix.get(i, j));
            }
        }
        matrix.set_diags(diags)?;
        Ok(matrix)
    }

    /// Creates a new matrix from a relaxed PHYLIP format file.
    ///
    /// Accepts full, lower-triangular-with-diagonal, or lower-triangular-without-diagonal
    /// layouts. The optional first line may declare the sequence count; if absent, it is
    /// inferred from the data rows.
    pub fn from_relaxed_phylip(infile: &str) -> anyhow::Result<Self> {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Layout {
            Full,
            LowerWithDiagonal,
            LowerWithoutDiagonal,
        }

        let mut rows: Vec<(String, Vec<f32>)> = Vec::new();
        let mut declared_size: Option<usize> = None;
        let mut seen = HashSet::new();

        let reader = crate::reader(infile)?;
        let mut lines = reader.lines();

        // Read the optional sequence count line
        if let Some(line_res) = lines.next() {
            let line = line_res?;
            if let Ok(size) = line.trim().parse::<usize>() {
                declared_size = Some(size);
            } else if let Some((name, values)) = Self::process_phylip_line(&line)? {
                if !seen.insert(name.clone()) {
                    anyhow::bail!("duplicate sequence name in PHYLIP matrix: {}", name);
                }
                rows.push((name, values));
            }
        }

        // Process remaining lines
        for line in lines {
            let line = line?;
            if let Some((name, values)) = Self::process_phylip_line(&line)? {
                if !seen.insert(name.clone()) {
                    anyhow::bail!("duplicate sequence name in PHYLIP matrix: {}", name);
                }
                rows.push((name, values));
            }
        }

        let size = declared_size.unwrap_or(rows.len());
        if rows.len() != size {
            anyhow::bail!(
                "PHYLIP matrix declares {} sequences but found {}",
                declared_size.unwrap_or(0),
                rows.len()
            );
        }

        if size == 0 {
            return Self::new(Vec::new());
        }

        // Infer the matrix layout from the first data row.
        let first_count = rows[0].1.len();
        let layout = if first_count >= size {
            Layout::Full
        } else if first_count >= 1 {
            Layout::LowerWithDiagonal
        } else {
            Layout::LowerWithoutDiagonal
        };

        // Validate row lengths and warn about extra values.
        for (i, (name, values)) in rows.iter().enumerate() {
            let expected = match layout {
                Layout::Full => size,
                Layout::LowerWithDiagonal => i + 1,
                Layout::LowerWithoutDiagonal => i,
            };
            if values.len() < expected {
                anyhow::bail!(
                    "malformed PHYLIP line for '{}': expected {} value(s), found {}",
                    name,
                    expected,
                    values.len()
                );
            }
            if values.len() > expected {
                log::warn!(
                    "line for '{}' contains {} extra value(s); ignoring values beyond the expected {} value(s) for {:?} layout",
                    name,
                    values.len() - expected,
                    expected,
                    layout
                );
            }
        }

        let names: Vec<String> = rows.iter().map(|(n, _)| n.clone()).collect();
        let mut matrix = Self::new(names)?;
        let mut diags = vec![0.0f32; size];

        // Fill the matrix from the lower-triangle portion of each row.
        for (i, (_name, values)) in rows.iter().enumerate() {
            match layout {
                Layout::Full | Layout::LowerWithDiagonal => {
                    for (j, &value) in values.iter().enumerate().take(i + 1) {
                        if j == i {
                            diags[i] = value;
                        } else {
                            matrix.set(i, j, value);
                        }
                    }
                }
                Layout::LowerWithoutDiagonal => {
                    for (j, &value) in values.iter().enumerate().take(i) {
                        matrix.set(i, j, value);
                    }
                }
            }
        }

        // Validate symmetry for full matrices.
        if layout == Layout::Full {
            for (i, (name, values)) in rows.iter().enumerate() {
                for (j, &value) in
                    values.iter().enumerate().skip(i + 1).take(size - i - 1)
                {
                    let expected = matrix.get(i, j);
                    if (value - expected).abs() > 1e-6 {
                        anyhow::bail!(
                            "asymmetric PHYLIP matrix at ('{}', '{}'): {} vs {}",
                            name,
                            rows[j].0,
                            value,
                            expected
                        );
                    }
                }
            }
        }

        matrix.set_diags(diags)?;
        Ok(matrix)
    }

    /// Parse a single non-empty PHYLIP data line into `(name, raw_values)`.
    /// Returns `Ok(None)` for empty or whitespace-only lines.
    fn process_phylip_line(line: &str) -> anyhow::Result<Option<(String, Vec<f32>)>> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(None);
        }

        let name = parts[0].to_string();
        let values: Vec<f32> = parts[1..]
            .iter()
            .map(|&s| {
                s.parse::<f32>()
                    .map_err(|e| anyhow::anyhow!("parse error: {e}"))
            })
            .collect::<anyhow::Result<Vec<f32>>>()?;

        Ok(Some((name, values)))
    }
}
