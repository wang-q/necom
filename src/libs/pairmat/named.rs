use std::collections::HashSet;
use std::io::BufRead;

use indexmap::{IndexMap, IndexSet};

use super::condensed::{get_condensed_index, CondensedMatrix};
use super::MatrixView;

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

    /// Create a matrix with numeric names ("0", "1", ..., "size-1").
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
    /// Returns 0.0 if either index is out of bounds.
    pub fn get(&self, row: usize, col: usize) -> f32 {
        if row >= self.size() || col >= self.size() {
            return 0.0;
        }
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
    /// Does nothing if either index is out of bounds.
    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        if row >= self.size() || col >= self.size() {
            return;
        }
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
    ///
    /// Constructs the underlying `CondensedMatrix` directly instead of going through
    /// an intermediate `ScoringMatrix`, reducing peak memory for large inputs.
    /// Parsed entries are stored as numeric indices rather than strings, and off-diagonal
    /// duplicate detection uses a dense `Vec<bool>` instead of a hash set, so dense inputs
    /// only pay for the matrix itself plus one byte per pair during construction.
    pub fn from_pair_scores(
        infile: &str,
        same: f32,
        missing: f32,
    ) -> anyhow::Result<Self> {
        let mut names = IndexSet::new();
        let mut entries: Vec<(usize, usize, f32)> = Vec::new();

        let reader = crate::reader(infile)?;
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
            if fields.len() < 3 {
                log::warn!(
                    "skipping malformed pairwise line (expected 3 tab-separated fields): {}",
                    line
                );
                continue;
            }
            if fields.len() > 3 {
                log::warn!(
                    "pairwise line contains {} extra column(s); ignoring: {}",
                    fields.len() - 3,
                    line
                );
            }

            let n1 = fields[0].to_string();
            let n2 = fields[1].to_string();
            if n1.is_empty() || n2.is_empty() {
                log::warn!("skipping pairwise line with empty sequence name: {}", line);
                continue;
            }
            let score: f32 = match fields[2].parse() {
                Ok(v) => v,
                Err(e) => {
                    log::warn!(
                        "skipping pairwise line with invalid score ({}): {}",
                        e,
                        line
                    );
                    continue;
                }
            };

            let (i1, _) = names.insert_full(n1);
            let (i2, _) = names.insert_full(n2);
            entries.push((i1, i2, score));
        }

        let size = names.len();
        let len = if size == 0 { 0 } else { size * (size - 1) / 2 };
        let mut matrix = CondensedMatrix::from_vec(size, vec![missing; len])?;
        let mut diags = vec![same; size];
        let mut seen_pairs: Vec<bool> = vec![false; len];
        let mut seen_self_pairs: HashSet<usize> = HashSet::new();

        for (i1, i2, score) in entries {
            if i1 == i2 {
                if !seen_self_pairs.insert(i1) {
                    let existing = diags[i1];
                    // Use total_cmp so NaN == NaN is treated as equal (no spurious
                    // conflict warning), while NaN vs finite still warns.
                    if existing.total_cmp(&score) != std::cmp::Ordering::Equal {
                        let name = names.get_index(i1).expect("valid self-pair index");
                        log::warn!(
                            "conflicting pairwise entry for ({}, {}): existing {} vs new {}; using last value",
                            name,
                            name,
                            existing,
                            score
                        );
                    }
                }
                diags[i1] = score;
                continue;
            }
            let (row, col) = if i1 < i2 { (i1, i2) } else { (i2, i1) };
            let idx = get_condensed_index(size, row, col);
            if seen_pairs[idx] {
                let existing = matrix.get(row, col);
                if existing.total_cmp(&score) != std::cmp::Ordering::Equal {
                    let name1 = names.get_index(row).expect("valid pair index");
                    let name2 = names.get_index(col).expect("valid pair index");
                    log::warn!(
                        "conflicting pairwise entry for ({}, {}): existing {} vs new {}; using last value",
                        name1,
                        name2,
                        existing,
                        score
                    );
                }
            }
            seen_pairs[idx] = true;
            matrix.set(row, col, score);
        }

        let names_map = build_name_map(names.into_iter().collect())?;
        Ok(NamedMatrix {
            names: names_map,
            matrix,
            diags: Some(diags),
        })
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
            match declared_size {
                Some(d) => anyhow::bail!(
                    "PHYLIP matrix declares {} sequences but found {} data rows. \
                     If the first line is a sequence name rather than the count, \
                     add an explicit count header before the first data row",
                    d,
                    rows.len()
                ),
                None => anyhow::bail!(
                    "could not infer PHYLIP size: expected {} data rows, found {}",
                    size,
                    rows.len()
                ),
            }
        }

        // Warn when a single-integer header happens to match the number of data
        // rows and the first sequence name is also numeric. In that case the
        // first line could have been a numeric sequence name (with no values),
        // so the caller may have intended an unheaded lower-triangular matrix.
        if declared_size == Some(rows.len())
            && !rows.is_empty()
            && rows[0].0.parse::<usize>().is_ok()
        {
            log::warn!(
                "Ambiguous PHYLIP input: the first line ({}) is interpreted as the \
                 sequence-count header because it is a single integer. If it was meant \
                 to be a sequence name, add an explicit count header before the first data row",
                declared_size.unwrap()
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
        // NaN requires special handling: `(NaN - x).abs() > 1e-6` evaluates to
        // `false` (NaN comparisons are always false), so a one-sided NaN would
        // silently pass the check. An explicit NaN-mismatch test catches this.
        if layout == Layout::Full {
            for (i, (name, values)) in rows.iter().enumerate() {
                for (j, &value) in
                    values.iter().enumerate().skip(i + 1).take(size - i - 1)
                {
                    let expected = matrix.get(i, j);
                    if value.is_nan() != expected.is_nan()
                        || (value - expected).abs() > 1e-6
                    {
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

impl MatrixView<f32> for NamedMatrix {
    fn size(&self) -> usize {
        self.size()
    }

    fn get(&self, row: usize, col: usize) -> f32 {
        self.get(row, col)
    }
}
