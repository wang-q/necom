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
    let m = CondensedMatrix::from_vec(3, data).unwrap();
    assert_eq!(m.get(0, 1), 1.0);
    assert_eq!(m.get(0, 2), 2.0);
    assert_eq!(m.get(1, 2), 3.0);
}

#[test]
fn test_condensed_matrix_from_vec_invalid_len() {
    let result = CondensedMatrix::from_vec(3, vec![1.0, 2.0]);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("Data length 2 does not match expected length 3 for size 3"));
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
    let transformed =
        super::transform_matrix(&matrix, "log", 1.0, 1.0, 0.0, false).unwrap();

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
    let transformed =
        super::transform_matrix(&matrix, "sqrt", 1.0, 1.0, 0.0, false).unwrap();

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
    let transformed =
        super::transform_matrix(&matrix, "inv-linear", 1.0, 1.0, 0.0, false).unwrap();

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

    let matrix =
        NamedMatrix::from_pair_scores(tmp.path().to_str().unwrap(), 0.0, 1.0).unwrap();
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

    let matrix =
        NamedMatrix::from_pair_scores(tmp.path().to_str().unwrap(), 0.0, 1.0).unwrap();
    assert_eq!(matrix.get_by_name("A", "B"), Some(0.5));
}

#[test]
fn test_from_relaxed_phylip_asymmetric() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Asymmetric full 3x3 matrix: upper triangle differs from lower triangle.
    writeln!(tmp, "3").unwrap();
    writeln!(tmp, "A 0.0 0.1 0.2").unwrap();
    writeln!(tmp, "B 0.1 0.0 0.999").unwrap();
    writeln!(tmp, "C 0.2 0.3 0.0").unwrap();

    let result = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap());
    assert!(
        result.is_err(),
        "asymmetric PHYLIP matrix should return an error"
    );
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("asymmetric"));
}

#[test]
fn test_from_relaxed_phylip_lower_no_diag() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Lower-triangular matrix without diagonal values.
    writeln!(tmp, "4").unwrap();
    writeln!(tmp, "A").unwrap();
    writeln!(tmp, "B 0.1").unwrap();
    writeln!(tmp, "C 0.2 0.3").unwrap();
    writeln!(tmp, "D 0.4 0.5 0.6").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    assert_eq!(matrix.get_by_name("A", "B"), Some(0.1));
    assert_eq!(matrix.get_by_name("A", "C"), Some(0.2));
    assert_eq!(matrix.get_by_name("A", "D"), Some(0.4));
    assert_eq!(matrix.get_by_name("B", "C"), Some(0.3));
    assert_eq!(matrix.get_by_name("B", "D"), Some(0.5));
    assert_eq!(matrix.get_by_name("C", "D"), Some(0.6));
    assert_eq!(matrix.get_by_name("A", "A"), Some(0.0));
    assert_eq!(matrix.get_by_name("C", "C"), Some(0.0));
}

#[test]
fn test_from_relaxed_phylip_lower_no_diag_no_header() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Lower-triangular matrix without header and without diagonal values.
    writeln!(tmp, "A").unwrap();
    writeln!(tmp, "B 0.1").unwrap();
    writeln!(tmp, "C 0.2 0.3").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    assert_eq!(matrix.size(), 3);
    assert_eq!(matrix.get_by_name("A", "B"), Some(0.1));
    assert_eq!(matrix.get_by_name("A", "C"), Some(0.2));
    assert_eq!(matrix.get_by_name("B", "C"), Some(0.3));
}

#[test]
fn test_matrix_format_from_mode_invalid() {
    match super::output::MatrixFormat::from_mode("unknown") {
        Err(e) => {
            let msg = format!("{}", e);
            assert!(
                msg.contains("unknown"),
                "error message should contain the invalid format"
            );
        }
        Ok(_) => panic!("invalid format should return an error"),
    }
}

#[test]
fn test_from_relaxed_phylip_basic_full_matrix() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "3").unwrap();
    writeln!(tmp, "A 0.0 1.0 2.0").unwrap();
    writeln!(tmp, "B 1.0 0.0 3.0").unwrap();
    writeln!(tmp, "C 2.0 3.0 0.0").unwrap();
    tmp.flush().unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    assert_eq!(matrix.get_names(), vec!["A", "B", "C"]);
    assert_eq!(matrix.get_by_name("A", "B"), Some(1.0));
    assert_eq!(matrix.get_by_name("A", "C"), Some(2.0));
    assert_eq!(matrix.get_by_name("B", "C"), Some(3.0));
}

#[test]
fn test_transform_linear() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Full 2x2: off-diag = 0.5, diags = 1.0
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "A 1.0 0.5").unwrap();
    writeln!(tmp, "B 0.5 1.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    // val = val * scale + offset; scale=2.0, offset=1.0
    let transformed =
        super::transform_matrix(&matrix, "linear", 1.0, 2.0, 1.0, false).unwrap();

    // Off-diagonal: 0.5 * 2.0 + 1.0 = 2.0
    assert_eq!(transformed.get(0, 1), 2.0);
    // Diagonal: 1.0 * 2.0 + 1.0 = 3.0
    assert_eq!(transformed.get(0, 0), 3.0);
    assert_eq!(transformed.get(1, 1), 3.0);
}

#[test]
fn test_transform_exp() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Full 2x2: off-diag = 0.5, diags = 0.0
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "A 0.0 0.5").unwrap();
    writeln!(tmp, "B 0.5 0.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    let transformed =
        super::transform_matrix(&matrix, "exp", 1.0, 1.0, 0.0, false).unwrap();

    // Off-diagonal: exp(-0.5)
    assert!((transformed.get(0, 1) - (-0.5f32).exp()).abs() < 1e-6);
    // Diagonal: exp(-0.0) = 1.0
    assert_eq!(transformed.get(0, 0), 1.0);
    assert_eq!(transformed.get(1, 1), 1.0);
}

#[test]
fn test_transform_square() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Full 2x2: off-diag = 0.5, diags = 2.0
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "A 2.0 0.5").unwrap();
    writeln!(tmp, "B 0.5 2.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    let transformed =
        super::transform_matrix(&matrix, "square", 1.0, 1.0, 0.0, false).unwrap();

    // Off-diagonal: 0.5 * 0.5 = 0.25
    assert_eq!(transformed.get(0, 1), 0.25);
    // Diagonal: 2.0 * 2.0 = 4.0
    assert_eq!(transformed.get(0, 0), 4.0);
    assert_eq!(transformed.get(1, 1), 4.0);
}

#[test]
fn test_transform_normalize_with_linear() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Full 3x3 with distinct diagonals; one diagonal is zero to exercise the <=1e-9 branch.
    // d_A = 4.0, d_B = 9.0, d_C = 0.0
    // off-diag: A-B = 6.0, A-C = 0.0, B-C = 5.0
    writeln!(tmp, "3").unwrap();
    writeln!(tmp, "A 4.0 6.0 0.0").unwrap();
    writeln!(tmp, "B 6.0 9.0 5.0").unwrap();
    writeln!(tmp, "C 0.0 5.0 0.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    // normalize then linear with scale=2.0, offset=1.0
    let transformed =
        super::transform_matrix(&matrix, "linear", 1.0, 2.0, 1.0, true).unwrap();

    // A-B: 6.0 / sqrt(4*9) = 6.0/6.0 = 1.0 -> 1.0*2.0+1.0 = 3.0
    assert_eq!(transformed.get(0, 1), 3.0);
    // A-C: d_C = 0.0 -> branch sets val = 0.0 -> 0.0*2.0+1.0 = 1.0
    assert_eq!(transformed.get(0, 2), 1.0);
    // B-C: 5.0 / sqrt(9*0.0) -> d_C = 0.0 branch -> val = 0.0 -> 1.0
    assert_eq!(transformed.get(1, 2), 1.0);
    // Diagonal: A: d=4.0>1e-9 -> 1.0 -> 3.0
    assert_eq!(transformed.get(0, 0), 3.0);
    // Diagonal: B: d=9.0>1e-9 -> 1.0 -> 3.0
    assert_eq!(transformed.get(1, 1), 3.0);
    // Diagonal: C: d=0.0 -> 0.0 -> 0.0*2.0+1.0 = 1.0
    assert_eq!(transformed.get(2, 2), 1.0);
}

#[test]
fn test_empty_matrix_boundary() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "0").unwrap();
    tmp.flush().unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    assert_eq!(matrix.size(), 0);
    assert!(matrix.get_names().is_empty());

    // write_phylip_matrix should emit just the count line "0\n".
    let mut buf = Vec::new();
    super::output::write_phylip_matrix(
        &matrix,
        super::output::MatrixFormat::Full,
        Some(6),
        &mut buf,
    )
    .unwrap();
    assert_eq!(b"0\n", buf.as_slice());
}

#[test]
fn test_write_subset_precision() {
    let names = vec!["A".to_string(), "B".to_string()];
    let mut m = NamedMatrix::new(names).unwrap();
    m.set(0, 1, 0.5);

    let mut buf = Vec::new();
    super::output::write_subset(
        &m,
        &["A".to_string(), "B".to_string()],
        Some(6),
        &mut buf,
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output, "2\nA\t0.000000\t0.500000\nB\t0.500000\t0.000000\n");
}

#[test]
fn test_single_element_matrix_boundary() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "1").unwrap();
    writeln!(tmp, "A 0.0").unwrap();
    tmp.flush().unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    assert_eq!(matrix.size(), 1);
    assert_eq!(matrix.get_names(), vec!["A"]);

    // No off-diagonal elements; get(0,0) returns the stored diagonal.
    assert_eq!(matrix.get(0, 0), 0.0);

    // write_phylip_matrix Full: "1\nA\t0.000000\n"
    let mut buf = Vec::new();
    super::output::write_phylip_matrix(
        &matrix,
        super::output::MatrixFormat::Full,
        Some(6),
        &mut buf,
    )
    .unwrap();
    let expected = "1\nA\t0.000000\n";
    assert_eq!(expected.as_bytes(), buf.as_slice());
}
