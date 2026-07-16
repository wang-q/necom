use super::condensed::{get_condensed_index, CondensedMatrix};
use super::named::NamedMatrix;
use super::scoring::ScoringMatrix;

#[test]
fn test_condensed_matrix_indexing() {
    // N=4
    // (0,1) -> 0
    // (0,2) -> 1
    // (0,3) -> 2
    // (1,2) -> 3
    // (1,3) -> 4
    // (2,3) -> 5
    let m = CondensedMatrix::new(4);
    assert_eq!(get_condensed_index(m.size(), 0, 1), 0);
    assert_eq!(get_condensed_index(m.size(), 0, 2), 1);
    assert_eq!(get_condensed_index(m.size(), 0, 3), 2);
    assert_eq!(get_condensed_index(m.size(), 1, 2), 3);
    assert_eq!(get_condensed_index(m.size(), 1, 3), 4);
    assert_eq!(get_condensed_index(m.size(), 2, 3), 5);
}

#[test]
fn test_condensed_matrix_rw() {
    let mut m = CondensedMatrix::new(3);
    m.set(0, 1, 1.0);
    m.set(2, 0, 2.0); // set (0,2) via swap
    m.set(1, 2, 3.0);

    assert_eq!(m.get(0, 1), 1.0);
    assert_eq!(m.get(1, 0), 1.0);
    assert_eq!(m.get(0, 2), 2.0);
    assert_eq!(m.get(2, 0), 2.0);
    assert_eq!(m.get(1, 2), 3.0);
    assert_eq!(m.get(0, 0), 0.0);

    // Test underlying data access
    let data = m.data();
    assert_eq!(data.len(), 3); // 3*2/2 = 3
                               // Order: (0,1), (0,2), (1,2) -> 1.0, 2.0, 3.0
    assert_eq!(data[0], 1.0);
    assert_eq!(data[1], 2.0);
    assert_eq!(data[2], 3.0);
}

#[test]
fn test_condensed_matrix_from_vec() {
    let data = vec![1.0, 2.0, 3.0];
    let m = CondensedMatrix::from_vec(3, data);
    assert_eq!(m.get(0, 1), 1.0);
    assert_eq!(m.get(0, 2), 2.0);
    assert_eq!(m.get(1, 2), 3.0);
}

#[test]
#[should_panic(expected = "Data length 2 does not match expected length 3 for size 3")]
fn test_condensed_matrix_from_vec_invalid_len() {
    CondensedMatrix::from_vec(3, vec![1.0, 2.0]);
}

#[test]
fn test_scoring_matrix_basic() {
    let mut m = ScoringMatrix::with_defaults(0.0, -1.0);
    m.set(0, 1, 5.0);
    m.set(2, 1, 10.0);

    // Check set values (symmetric)
    assert_eq!(m.get(0, 1), 5.0);
    assert_eq!(m.get(1, 0), 5.0);
    assert_eq!(m.get(1, 2), 10.0);

    // Check diagonal default
    assert_eq!(m.get(0, 0), 0.0);
    assert_eq!(m.get(3, 3), 0.0);

    // Check missing default
    assert_eq!(m.get(0, 2), -1.0);
    assert_eq!(m.get(3, 4), -1.0);
}

#[test]
fn test_named_matrix_basic() {
    let names = vec!["A".to_string(), "B".to_string()];
    let mut m = NamedMatrix::new(names).unwrap();

    m.set(0, 1, 0.5);
    assert_eq!(m.get(0, 1), 0.5);
    assert_eq!(m.get(1, 0), 0.5);
    assert_eq!(m.get(0, 0), 0.0);

    assert_eq!(m.get_by_name("A", "B"), Some(0.5));
}

#[test]
fn test_named_matrix_indexing() {
    let names = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    let m = NamedMatrix::new(names).unwrap();

    // Size 3 -> len 3
    assert_eq!(m.values().len(), 3);

    // Index check
    // (0,1) -> 0
    // (0,2) -> 1
    // (1,2) -> 2
    assert_eq!(m.index(0, 1), 0);
    assert_eq!(m.index(0, 2), 1);
    assert_eq!(m.index(1, 2), 2);
}

#[test]
fn test_set_diags_wrong_length() {
    let names = vec!["A".to_string(), "B".to_string()];
    let mut m = NamedMatrix::new(names).unwrap();
    assert!(m.set_diags(vec![1.0]).is_err());
    assert!(m.set_diags(vec![1.0, 2.0]).is_ok());
}

#[test]
fn test_from_relaxed_phylip_malformed() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "A 0.0").unwrap();
    writeln!(tmp, "B").unwrap(); // missing required value

    let result = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap());
    assert!(
        result.is_err(),
        "malformed PHYLIP input should return an error"
    );
}

#[test]
fn test_transform_log_non_positive() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Lower-triangle PHYLIP: A diag=0.0; B off-diag=-1.0, diag=0.0
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "A 0.0").unwrap();
    writeln!(tmp, "B -1.0 0.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    let transformed = super::transform_matrix(&matrix, "log", 1.0, 1.0, 0.0, false).unwrap();

    assert_eq!(transformed.get(0, 1), f32::INFINITY);
    assert_eq!(transformed.get(0, 0), 0.0);
    assert_eq!(transformed.get(1, 1), 0.0);
}

#[test]
fn test_transform_sqrt_negative() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "A -4.0").unwrap();
    writeln!(tmp, "B -1.0 -9.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    let transformed = super::transform_matrix(&matrix, "sqrt", 1.0, 1.0, 0.0, false).unwrap();

    assert!(transformed.get(0, 1).is_nan());
    assert!(transformed.get(0, 0).is_nan());
    assert!(transformed.get(1, 1).is_nan());
}

#[test]
fn test_from_relaxed_phylip_truncated() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // 3 sequences but only 3 lower-triangle values (expected 6)
    writeln!(tmp, "3").unwrap();
    writeln!(tmp, "A 0.0").unwrap();
    writeln!(tmp, "B 0.1 0.0").unwrap();

    let result = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap());
    assert!(
        result.is_err(),
        "truncated PHYLIP matrix should return an error"
    );
}

#[test]
fn test_from_relaxed_phylip_duplicate_name() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "A 0.0").unwrap();
    writeln!(tmp, "A 0.1 0.0").unwrap();

    let result = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap());
    assert!(
        result.is_err(),
        "duplicate sequence name should return an error"
    );
}

#[test]
fn test_transform_inv_linear_diagonal() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Full matrix with non-zero diagonal
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "A 1.0 0.5").unwrap();
    writeln!(tmp, "B 0.5 1.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    let transformed = super::transform_matrix(&matrix, "inv-linear", 1.0, 1.0, 0.0, false).unwrap();

    // Off-diagonal: 1.0 - 0.5 = 0.5
    assert_eq!(transformed.get(0, 1), 0.5);
    // Diagonal should stay 0 for a valid distance matrix
    assert_eq!(transformed.get(0, 0), 0.0);
    assert_eq!(transformed.get(1, 1), 0.0);
}

#[test]
fn test_named_matrix_duplicate_name() {
    let names = vec!["A".to_string(), "A".to_string()];
    let result = NamedMatrix::new(names);
    assert!(
        result.is_err(),
        "duplicate sequence name should return an error"
    );
}

#[test]
fn test_from_pair_scores_duplicate_pair_uses_last_value() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Duplicate pair (A,B) with different values; last value should win.
    writeln!(tmp, "A\tB\t0.1").unwrap();
    writeln!(tmp, "A\tB\t0.9").unwrap();

    let matrix = NamedMatrix::from_pair_scores(tmp.path().to_str().unwrap(), 0.0, 1.0).unwrap();
    assert_eq!(matrix.get_by_name("A", "B"), Some(0.9));
}

#[test]
fn test_from_relaxed_phylip_extra_values() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Full 3x3 matrix: upper-triangle values beyond the lower triangle are ignored.
    writeln!(tmp, "3").unwrap();
    writeln!(tmp, "A 0.0 0.1 0.2").unwrap();
    writeln!(tmp, "B 0.1 0.0 0.3").unwrap();
    writeln!(tmp, "C 0.2 0.3 0.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    assert_eq!(matrix.get_by_name("A", "B"), Some(0.1));
    assert_eq!(matrix.get_by_name("A", "C"), Some(0.2));
    assert_eq!(matrix.get_by_name("B", "C"), Some(0.3));
}

#[test]
fn test_from_pair_scores_extra_columns() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Pairwise line with extra columns beyond name1, name2, distance.
    writeln!(tmp, "A\tB\t0.5\textra\tdata").unwrap();

    let matrix = NamedMatrix::from_pair_scores(tmp.path().to_str().unwrap(), 0.0, 1.0).unwrap();
    assert_eq!(matrix.get_by_name("A", "B"), Some(0.5));
}
