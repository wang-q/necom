//! A *symmetric* scoring matrix to be used for clustering.
mod condensed;
mod named;
mod output;
mod scoring;
mod transform;

pub use condensed::{get_condensed_index, CondensedMatrix};
pub use named::NamedMatrix;
pub use output::{
    extract_common_lower_triangle, write_phylip_matrix, write_subset, MatrixFormat,
};
pub use scoring::ScoringMatrix;
pub use transform::transform_matrix;

/// A read-only view of a square symmetric matrix.
///
/// Provides the minimal interface required by clustering algorithms:
/// matrix size and symmetric element access.
pub trait MatrixView<T = f32>
where
    T: Copy + PartialOrd,
{
    /// Number of rows/columns in the matrix.
    fn size(&self) -> usize;

    /// Get the value at `(row, col)`.
    ///
    /// The matrix is symmetric, so `get(row, col)` equals `get(col, row)`.
    fn get(&self, row: usize, col: usize) -> T;
}

#[cfg(test)]
mod tests;
