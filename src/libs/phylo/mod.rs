//! Phylogenetic tree library: Newick parsing, manipulation, comparison, and I/O.

/// Tree comparison algorithms.
pub mod cmp;
/// Error types for tree operations.
pub mod error;
/// Node representation and ID type.
pub mod node;
/// Newick parser and label utilities.
pub mod parser;
/// Taxonomy table parsing helpers.
pub mod taxonomy;
/// Tree structure, traversal, and algorithms.
pub mod tree;

/// Tree comparison trait (RF, WRF, KF distances and splits).
pub use cmp::TreeComparison;
/// Tree-level error type.
pub use error::TreeError;
/// Node type and its lightweight ID.
pub use node::{Node, NodeId};
/// Newick label sanitizer.
pub use parser::newick_safe;
/// Taxonomy table parser and type.
pub use taxonomy::{read_taxonomy, TaxonomyTable};
/// Arena-based phylogenetic tree.
pub use tree::Tree;
