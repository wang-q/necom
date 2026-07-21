use std::collections::HashMap;
use std::io::BufRead;

use super::MatrixView;

/// Linear index into the lower triangle of an N×N matrix **including** the diagonal.
///
/// Assumes `i <= j`. The index is independent of `N`, so entries inserted before
/// the matrix size is known remain valid after the inferred size grows.
fn condensed_index_with_diag(i: usize, j: usize) -> usize {
    debug_assert!(i <= j, "condensed_index_with_diag requires i <= j");
    j * (j + 1) / 2 + i
}

/// A symmetric scoring matrix parameterized over the value type `T`.
///
/// Internally stores only the lower triangle (including the diagonal) using a
/// single compressed `usize` key, reducing per-entry overhead compared to a
/// `(usize, usize)` tuple key.
#[derive(Debug, Clone)]
pub struct ScoringMatrix<T> {
    size: Option<usize>,
    same: Option<T>,
    missing: Option<T>,
    data: HashMap<usize, T>,
    max_index: usize,
}

impl<T> Default for ScoringMatrix<T>
where
    T: Default + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ScoringMatrix<T>
where
    T: Default + Copy,
{
    /// Creates a new empty matrix with default values.
    ///
    /// ```
    /// # use necom::libs::pairmat::ScoringMatrix;
    /// let matrix: ScoringMatrix<i32> = ScoringMatrix::new();
    /// assert_eq!(matrix.get(0, 0), 0);  // Using T::default()
    /// ```
    pub fn new() -> Self {
        ScoringMatrix {
            size: None,
            same: None,
            missing: None,
            data: HashMap::new(),
            max_index: 0,
        }
    }

    /// Creates a new matrix with specified default values.
    ///
    /// ```
    /// # use necom::libs::pairmat::ScoringMatrix;
    /// let matrix = ScoringMatrix::with_defaults(0.0, -1.0);
    /// assert_eq!(matrix.get(0, 0), 0.0);    // same value
    /// assert_eq!(matrix.get(0, 1), -1.0);   // missing value
    /// ```
    pub fn with_defaults(same: T, missing: T) -> Self {
        ScoringMatrix {
            size: None,
            same: Some(same),
            missing: Some(missing),
            data: HashMap::new(),
            max_index: 0,
        }
    }

    /// Creates a new matrix with specified size and default values.
    ///
    /// ```
    /// # use necom::libs::pairmat::ScoringMatrix;
    /// let matrix = ScoringMatrix::with_size_and_defaults(3, 1.0, 0.0);
    /// assert_eq!(matrix.size(), 3);
    /// assert_eq!(matrix.get(0, 0), 1.0);    // same value
    /// assert_eq!(matrix.get(0, 1), 0.0);    // missing value
    /// ```
    pub fn with_size_and_defaults(size: usize, same: T, missing: T) -> Self {
        ScoringMatrix {
            size: Some(size),
            same: Some(same),
            missing: Some(missing),
            data: HashMap::new(),
            max_index: 0,
        }
    }

    /// Creates a new matrix with a fixed size but no default `same`/`missing` values.
    pub fn with_size(size: usize) -> Self {
        ScoringMatrix {
            size: Some(size),
            same: None,
            missing: None,
            data: HashMap::new(),
            max_index: 0,
        }
    }

    /// Effective matrix size: the declared size if set, otherwise inferred from
    /// the largest index seen by `set`.
    pub fn size(&self) -> usize {
        self.size.unwrap_or(self.max_index + 1)
    }

    /// Sets a fixed size for the matrix.
    ///
    /// # Panics
    ///
    /// Panics if entries have already been stored with indices larger than or
    /// equal to `size`, because those entries would become invisible to `get`.
    pub fn set_size(&mut self, size: usize) {
        assert!(
            self.max_index < size || self.data.is_empty(),
            "set_size({}) called but entries with index {} already exist",
            size,
            self.max_index
        );
        self.size = Some(size);
    }

    /// Stores `value` at `(row, col)`.
    ///
    /// The matrix is symmetric, so `set(row, col, v)` is equivalent to
    /// `set(col, row, v)`. If the matrix size has not been set, it is inferred
    /// from the largest column index seen so far (`max_index + 1`).
    ///
    /// When the size has been fixed via [`set_size`](Self::set_size) and either
    /// index is out of bounds, the write is silently ignored, matching the
    /// behavior of [`NamedMatrix::set`](super::NamedMatrix::set).
    pub fn set(&mut self, row: usize, col: usize, value: T) {
        let (i, j) = if row <= col { (row, col) } else { (col, row) };

        match self.size {
            // Fixed size: silently ignore out-of-bounds writes.
            Some(n) if i >= n || j >= n => return,
            // Fixed size, in bounds: proceed.
            Some(_) => {}
            // Dynamic size: infer from the largest index seen so far.
            None => self.max_index = self.max_index.max(j),
        }

        let key = condensed_index_with_diag(i, j);
        self.data.insert(key, value);
    }

    /// Gets the value at `(row, col)`.
    ///
    /// The matrix is symmetric, so `get(row, col)` equals `get(col, row)`.
    /// Returns the configured `same` value for diagonal elements and the
    /// `missing` value for off-diagonal elements that have not been explicitly
    /// set. If either index is out of bounds, the default value for that
    /// position is returned.
    pub fn get(&self, row: usize, col: usize) -> T {
        let n = self.size();
        if row >= n || col >= n {
            return if row == col {
                self.same.unwrap_or_default()
            } else {
                self.missing.unwrap_or_default()
            };
        }

        let (i, j) = if row <= col { (row, col) } else { (col, row) };
        let key = condensed_index_with_diag(i, j);
        if let Some(&value) = self.data.get(&key) {
            return value;
        }

        if row == col {
            self.same.unwrap_or_default()
        } else {
            self.missing.unwrap_or_default()
        }
    }
}

impl<T> MatrixView<T> for ScoringMatrix<T>
where
    T: Default + Copy,
{
    fn size(&self) -> usize {
        self.size()
    }

    fn get(&self, row: usize, col: usize) -> T {
        self.get(row, col)
    }
}

impl ScoringMatrix<f32> {
    /// Load a `ScoringMatrix<f32>` and ordered name list from a 3-column pairwise TSV.
    /// Self-pairs default to `same`; missing pairs default to `missing`.
    ///
    /// Parsed entries are stored as numeric indices rather than strings to keep
    /// peak memory low for large inputs.
    pub fn from_pair_scores(
        infile: &str,
        same: f32,
        missing: f32,
    ) -> anyhow::Result<(Self, Vec<String>)> {
        let mut names = indexmap::IndexSet::new();
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
        let mut matrix = Self::with_size_and_defaults(size, same, missing);

        for (i1, i2, score) in entries {
            let (i, j) = if i1 <= i2 { (i1, i2) } else { (i2, i1) };
            let key = condensed_index_with_diag(i, j);
            if let Some(&existing) = matrix.data.get(&key) {
                // Use total_cmp so NaN == NaN is treated as equal (no spurious
                // conflict warning), while NaN vs finite still warns.
                if existing.total_cmp(&score) != std::cmp::Ordering::Equal {
                    let name1 = names.get_index(i).expect("valid pair index");
                    let name2 = names.get_index(j).expect("valid pair index");
                    log::warn!(
                        "conflicting pairwise entry for ({}, {}): existing {} vs new {}; using last value",
                        name1,
                        name2,
                        existing,
                        score
                    );
                }
            }
            matrix.set(i1, i2, score);
        }

        Ok((matrix, names.into_iter().collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::condensed_index_with_diag;

    /// Alternative computation of the lower-triangular-with-diagonal index.
    ///
    /// Sums the number of stored elements in rows `0..j` (`j` elements before row `j`,
    /// each row `k` stores `k+1` elements) and adds `i`.
    fn expected_index(i: usize, j: usize) -> usize {
        assert!(i <= j, "expected_index requires i <= j");
        (0..j).map(|k| k + 1).sum::<usize>() + i
    }

    #[test]
    fn test_lower_triangular_index_correctness_n100() {
        const N: usize = 100;
        for j in 0..N {
            for i in 0..=j {
                let actual = condensed_index_with_diag(i, j);
                let expected = expected_index(i, j);
                assert_eq!(
                    actual, expected,
                    "index mismatch for ({}, {}): got {}, expected {}",
                    i, j, actual, expected
                );
            }
        }
    }

    #[test]
    fn test_lower_triangular_index_uniqueness_n100() {
        const N: usize = 100;
        let total = N * (N + 1) / 2;
        let mut seen = std::collections::HashSet::with_capacity(total);

        for j in 0..N {
            for i in 0..=j {
                let key = condensed_index_with_diag(i, j);
                assert!(
                    key < total,
                    "key {} for ({}, {}) is out of expected range [0, {})",
                    key,
                    i,
                    j,
                    total
                );
                assert!(seen.insert(key), "duplicate key {} for ({}, {})", key, i, j);
            }
        }

        assert_eq!(seen.len(), total, "expected {} unique keys", total);
    }

    #[test]
    fn test_lower_triangular_index_boundary_cases() {
        const N: usize = 100;

        // First diagonal element.
        assert_eq!(condensed_index_with_diag(0, 0), 0);

        // Last diagonal element.
        assert_eq!(condensed_index_with_diag(N - 1, N - 1), N * (N + 1) / 2 - 1);

        // First row, last column.
        assert_eq!(condensed_index_with_diag(0, N - 1), (N - 1) * N / 2);

        // Second-to-last row, last column.
        assert_eq!(
            condensed_index_with_diag(N - 2, N - 1),
            (N - 1) * N / 2 + (N - 2)
        );
    }

    #[test]
    fn test_lower_triangular_index_symmetry() {
        // Callers normalize (row, col) to (i, j) where i <= j before calling
        // condensed_index_with_diag. Verify both orderings produce the same key.
        let (row, col) = (3, 7);
        let (i, j) = if row <= col { (row, col) } else { (col, row) };
        let (i2, j2) = if col <= row { (col, row) } else { (row, col) };
        assert_eq!(
            condensed_index_with_diag(i, j),
            condensed_index_with_diag(i2, j2)
        );
    }

    #[test]
    fn test_lower_triangular_index_monotonicity() {
        // Within the same row j, key increases with i.
        for j in 1..100 {
            for i in 1..=j {
                assert!(
                    condensed_index_with_diag(i, j)
                        > condensed_index_with_diag(i - 1, j),
                    "index should increase with i at row {}",
                    j
                );
            }
        }

        // For the same diagonal position, key increases with j.
        for j in 1..100 {
            assert!(
                condensed_index_with_diag(j, j)
                    > condensed_index_with_diag(j - 1, j - 1),
                "diagonal index should increase with j at {}",
                j
            );
        }
    }
}
