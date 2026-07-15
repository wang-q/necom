// Enable the unstable `portable_simd` feature (requires nightly).
// It is used by `libs::linalg` for SIMD-accelerated distance and similarity calculations.
#![feature(portable_simd)]

pub mod libs;
pub use libs::io::{read_lines, reader, writer};
