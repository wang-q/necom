use std::collections::HashMap;
use std::io::BufRead;

use super::MatrixView;

/// Triangular number `T(i) = i * (i - 1) / 2`.
///
/// Uses `saturating_sub` so that `T(0) == 0` without underflow on `usize`.
fn triangular_number(i: usize) -> usize {
    i * i.saturating_sub(1) / 2
}

/// Linear index into the upper triangle of an N×N matrix **including** the diagonal.
///
/// Assumes `i <= j` and `j < n`. The total number of stored elements is `n * (n + 1) / 2`.
fn upper_index_with_diag(n: usize, i: usize, j: usize) -> usize {
    debug_assert!(i <= j, "upper_index_with_diag requires i <= j");
    debug_assert!(j < n, "upper_index_with_diag requires j < n");
    i * n - triangular_number(i) + (j - i)
}

/// A symmetric scoring matrix parameterized over the value type `T`.
///
/// Internally stores only the upper triangle (including the diagonal) using a
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
    /// equal to `size`, because their compressed keys would no longer be valid.
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
    /// # Panics
    ///
    /// Panics if the size has been fixed via [`set_size`](Self::set_size) and
    /// either index is out of bounds, because the compressed key would no
    /// longer be valid.
    pub fn set(&mut self, row: usize, col: usize, value: T) {
        let (i, j) = if row <= col { (row, col) } else { (col, row) };
        self.max_index = self.max_index.max(j);

        let n = self.size.unwrap_or(self.max_index + 1);
        assert!(
            i < n && j < n,
            "ScoringMatrix::set({}, {}) out of bounds for size {}",
            row,
            col,
            n
        );
        let key = upper_index_with_diag(n, i, j);
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
        let key = upper_index_with_diag(n, i, j);
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
    T: Default + Copy + PartialOrd,
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
    pub fn from_pair_scores(
        infile: &str,
        same: f32,
        missing: f32,
    ) -> anyhow::Result<(Self, Vec<String>)> {
        let mut names = indexmap::IndexSet::new();
        let mut entries: Vec<(String, String, f32)> = Vec::new();

        let reader = crate::reader(infile)?;
        for line in reader.lines() {
            let line = line?;
            let fields: Vec<&str> = line.split('\t').collect();
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

            names.insert(n1.clone());
            names.insert(n2.clone());
            entries.push((n1, n2, score));
        }

        let size = names.len();
        let mut matrix = Self::with_size_and_defaults(size, same, missing);
        let name_to_index: indexmap::IndexMap<String, usize> = names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i))
            .collect();

        for (n1, n2, score) in entries {
            let i1 = name_to_index[&n1];
            let i2 = name_to_index[&n2];
            let (i, j) = if i1 <= i2 { (i1, i2) } else { (i2, i1) };
            let key = upper_index_with_diag(size, i, j);
            if let Some(&existing) = matrix.data.get(&key) {
                if existing != score {
                    log::warn!(
                        "conflicting pairwise entry for ({}, {}): existing {} vs new {}; using last value",
                        n1,
                        n2,
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
