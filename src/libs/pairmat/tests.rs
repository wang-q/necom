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
fn test_from_pair_scores_duplicate_self_pair_warns() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Duplicate self-pair (A,A) with different values; last value should win.
    writeln!(tmp, "A\tA\t0.1").unwrap();
    writeln!(tmp, "A\tA\t0.9").unwrap();

    let matrix =
        NamedMatrix::from_pair_scores(tmp.path().to_str().unwrap(), 0.0, 1.0).unwrap();
    assert_eq!(matrix.get_by_name("A", "A"), Some(0.9));
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
fn test_transform_normalize_negative_diags() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Full 2x2 with all-negative diagonals. Prior to the fix, `max_diag` was
    // computed with `fold(0.0f32, ...)` which masked negative values and
    // reported `max_diag == 0.0`. The fix uses `NEG_INFINITY` so the true max
    // is reported in the warning. Behaviorally, negative diagonals must be
    // treated like zero diagonals (the `<= 1e-9` branch zeros off-diagonals).
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "A -4.0 6.0").unwrap();
    writeln!(tmp, "B 6.0 -9.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    // normalize then linear with scale=2.0, offset=1.0
    let transformed =
        super::transform_matrix(&matrix, "linear", 1.0, 2.0, 1.0, true).unwrap();

    // Both diagonals are negative, so d_i <= 1e-9 triggers the zero branch:
    // off-diag val = 0.0 -> 0.0*2.0+1.0 = 1.0
    assert_eq!(transformed.get(0, 1), 1.0);
    // Diagonals: d <= 1e-9 -> 0.0 -> 0.0*2.0+1.0 = 1.0
    assert_eq!(transformed.get(0, 0), 1.0);
    assert_eq!(transformed.get(1, 1), 1.0);
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
fn test_write_subset_deduplicates_names() {
    let names = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    let mut m = NamedMatrix::new(names).unwrap();
    m.set(0, 1, 0.5);
    m.set(0, 2, 0.6);
    m.set(1, 2, 0.7);

    // Duplicate names should be kept once (first occurrence) and not produce
    // duplicate rows/columns in the output.
    let mut buf = Vec::new();
    super::output::write_subset(
        &m,
        &[
            "A".to_string(),
            "B".to_string(),
            "A".to_string(),
            "B".to_string(),
        ],
        Some(6),
        &mut buf,
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output, "2\nA\t0.000000\t0.500000\nB\t0.500000\t0.000000\n");
}

#[test]
fn test_write_phylip_strict_name_truncation_bytes() {
    let names = vec!["αβγδεζηθικλ".to_string(), "B".to_string()];
    let mut m = NamedMatrix::new(names).unwrap();
    m.set(0, 1, 0.5);

    let mut buf = Vec::new();
    super::output::write_phylip_matrix(
        &m,
        super::output::MatrixFormat::Strict,
        Some(6),
        &mut buf,
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    // First line is the count; second line starts with exactly 10 bytes of name.
    assert_eq!(lines[0], "2");
    let first_line = lines[1];
    let name_field = &first_line[..10];
    assert_eq!(name_field.len(), 10, "strict name field must be 10 bytes");
    assert!(name_field.starts_with("α"));
    // Full matrix row: "<name> 0.000000 0.500000" (diagonal then off-diagonal).
    assert_eq!(&first_line[10..], " 0.000000 0.500000");
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

#[test]
fn test_phylip_no_header_size_mismatch() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // No declared-size header; row B has too few values for the inferred layout.
    writeln!(tmp, "A 0.0").unwrap();
    writeln!(tmp, "B 0.1").unwrap();

    let result = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap());
    assert!(
        result.is_err(),
        "inconsistent no-header PHYLIP should error"
    );
    let msg = format!("{}", result.unwrap_err());
    assert!(
        !msg.contains("declares 0 sequences"),
        "unexpected 'declares 0 sequences' in error: {}",
        msg
    );
    assert!(
        msg.contains("malformed PHYLIP line"),
        "expected malformed line error, got: {}",
        msg
    );
}

#[test]
fn test_scoring_matrix_set_infers_size() {
    let mut m = ScoringMatrix::with_defaults(0.0, -1.0);
    m.set(2, 5, 7.0);

    // Size should be inferred from max_index + 1.
    assert_eq!(m.size(), 6);
    assert_eq!(m.get(2, 5), 7.0);
    assert_eq!(m.get(5, 2), 7.0);
}

#[test]
fn test_scoring_matrix_set_out_of_bounds_silently_ignored() {
    // Out-of-bounds writes on a fixed-size matrix are silently ignored,
    // matching NamedMatrix::set behavior.
    let mut m = ScoringMatrix::with_size_and_defaults(3, 0.0, -1.0);
    m.set(5, 0, 1.0);
    m.set(0, 5, 2.0);
    m.set(5, 5, 3.0);

    // Size and existing entries are unchanged.
    assert_eq!(m.size(), 3);
    assert_eq!(m.get(0, 0), 0.0); // same default
    assert_eq!(m.get(0, 1), -1.0); // missing default
}

#[test]
fn test_condensed_matrix_out_of_bounds() {
    let mut m = CondensedMatrix::new(3);
    m.set(0, 1, 0.5);

    // Out-of-bounds reads return 0.0 instead of panicking.
    assert_eq!(m.get(0, 5), 0.0);
    assert_eq!(m.get(5, 0), 0.0);
    assert_eq!(m.get(5, 5), 0.0);

    // Out-of-bounds writes are silently ignored.
    m.set(0, 5, 9.0);
    m.set(5, 0, 9.0);
    m.set(5, 5, 9.0);
    assert_eq!(m.get(0, 1), 0.5);
}

#[test]
fn test_named_matrix_out_of_bounds() {
    let names = vec!["A".to_string(), "B".to_string()];
    let mut m = NamedMatrix::new(names).unwrap();
    m.set(0, 1, 0.5);

    // Out-of-bounds reads return 0.0 instead of panicking.
    assert_eq!(m.get(0, 5), 0.0);
    assert_eq!(m.get(5, 0), 0.0);
    assert_eq!(m.get(5, 5), 0.0);

    // Out-of-bounds writes are silently ignored.
    m.set(0, 5, 9.0);
    m.set(5, 0, 9.0);
    m.set(5, 5, 9.0);
    assert_eq!(m.get(0, 1), 0.5);
}

#[test]
fn test_from_pair_scores_crlf() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // CRLF line endings with leading/trailing whitespace.
    tmp.write_all(b"A\tB\t0.1\r\n  B\tC\t0.2 \r\nC\tA\t0.3\r\n")
        .unwrap();

    let matrix =
        NamedMatrix::from_pair_scores(tmp.path().to_str().unwrap(), 0.0, 1.0).unwrap();
    assert_eq!(matrix.get_by_name("A", "B"), Some(0.1));
    assert_eq!(matrix.get_by_name("B", "C"), Some(0.2));
    assert_eq!(matrix.get_by_name("C", "A"), Some(0.3));
}

#[test]
fn test_from_pair_scores_empty_name() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Empty first name and whitespace-only second name should be skipped.
    writeln!(tmp, "\tB\t0.1").unwrap();
    writeln!(tmp, "A\t \t0.2").unwrap();
    writeln!(tmp, "A\tC\t0.3").unwrap();

    let matrix =
        NamedMatrix::from_pair_scores(tmp.path().to_str().unwrap(), 0.0, 1.0).unwrap();
    assert_eq!(matrix.size(), 2);
    assert_eq!(matrix.get_by_name("A", "C"), Some(0.3));
}

#[test]
fn test_scoring_matrix_from_pair_scores_crlf() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"A\tB\t0.1\r\nB\tC\t0.2\r\n").unwrap();

    let (matrix, names) =
        ScoringMatrix::from_pair_scores(tmp.path().to_str().unwrap(), 0.0, -1.0)
            .unwrap();
    assert_eq!(
        names,
        vec!["A".to_string(), "B".to_string(), "C".to_string()]
    );
    // A=0, B=1, C=2 based on insertion order.
    assert_eq!(matrix.get(0, 1), 0.1);
    assert_eq!(matrix.get(1, 2), 0.2);
    assert_eq!(matrix.get(0, 2), -1.0);
}

#[test]
fn test_scoring_matrix_from_pair_scores_empty_name() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "\tB\t0.1").unwrap();
    writeln!(tmp, "A\tC\t0.3").unwrap();

    let (matrix, names) =
        ScoringMatrix::from_pair_scores(tmp.path().to_str().unwrap(), 0.0, -1.0)
            .unwrap();
    assert_eq!(names, vec!["A".to_string(), "C".to_string()]);
    assert_eq!(matrix.get(0, 1), 0.3);
}

#[test]
fn test_from_relaxed_phylip_numeric_name_with_values() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // No header; the first sequence name is purely numeric but includes values,
    // so it must be parsed as a data row rather than a count header.
    writeln!(tmp, "123 0.0 0.5").unwrap();
    writeln!(tmp, "456 0.5 0.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    assert_eq!(matrix.size(), 2);
    assert_eq!(matrix.get_names(), vec!["123", "456"]);
    assert_eq!(matrix.get_by_name("123", "456"), Some(0.5));
}

#[test]
fn test_from_relaxed_phylip_numeric_name_after_header() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Explicit count header lets the first data row use a numeric name.
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "123 0.0 0.5").unwrap();
    writeln!(tmp, "456 0.5 0.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    assert_eq!(matrix.size(), 2);
    assert_eq!(matrix.get_names(), vec!["123", "456"]);
    assert_eq!(matrix.get_by_name("123", "456"), Some(0.5));
}

#[test]
fn test_from_relaxed_phylip_numeric_header_mismatch_error() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // Single integer first line is interpreted as a count header, but the file
    // does not contain that many data rows.
    writeln!(tmp, "123").unwrap();
    writeln!(tmp, "456 0.1").unwrap();
    writeln!(tmp, "789 0.2 0.3").unwrap();

    let result = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap());
    assert!(
        result.is_err(),
        "numeric-only first line with mismatched row count should error"
    );
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("add an explicit count header"),
        "error should suggest adding a count header, got: {}",
        msg
    );
}

#[test]
fn test_from_relaxed_phylip_numeric_header_ambiguity_warning() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    // The first line is a single integer that matches the row count, and the
    // first sequence name is also numeric. Parsing succeeds, but the caller
    // should be warned that the first line may have been a numeric name.
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "3 0.1").unwrap();
    writeln!(tmp, "4 0.2 0.0").unwrap();

    let matrix = NamedMatrix::from_relaxed_phylip(tmp.path().to_str().unwrap()).unwrap();
    assert_eq!(matrix.size(), 2);
    assert_eq!(matrix.get_names(), vec!["3", "4"]);
    assert_eq!(matrix.get_by_name("3", "4"), Some(0.2));
}

// ========================================================================
// SciPy parity tests for `CondensedMatrix` indexing.
//
// Mirrors `scipy.spatial.tests.test_distance::TestSquareForm` (line 1555-1619)
// and `TestNumObsY` (line 1622-1673). SciPy's `squareform` converts between
// condensed (upper-triangle vector) and full matrix representations; our
// `CondensedMatrix` uses the same condensed indexing. These tests verify the
// round-trip and the implicit `num_obs_y` behavior (inferring N from the
// condensed vector length via N*(N-1)/2 == len).
// ========================================================================

#[test]
fn test_scipy_squareform_roundtrip() {
    // SciPy `ytdist` condensed vector (15 values for N=6).
    let data = vec![
        662.0, 877.0, 255.0, 412.0, 996.0, 295.0, 468.0, 268.0, 400.0, 754.0, 564.0,
        138.0, 219.0, 869.0, 669.0,
    ];
    let n = 6;
    let m = CondensedMatrix::from_vec(n, data.clone()).unwrap();

    // Dimension invariants.
    assert_eq!(m.size(), n);
    assert_eq!(m.data().len(), n * (n - 1) / 2);

    // Round-trip: every (i, j) pair maps back to data[condensed_index(n, i, j)].
    for i in 0..n {
        // Diagonal is zero by contract.
        assert_eq!(m.get(i, i), 0.0);
        for j in (i + 1)..n {
            let idx = get_condensed_index(n, i, j);
            assert_eq!(m.get(i, j), data[idx]);
            // Symmetric access: get(j, i) must equal get(i, j).
            assert_eq!(m.get(j, i), m.get(i, j));
        }
    }
}

#[test]
fn test_scipy_num_obs_y() {
    // SciPy `num_obs_y` infers N from condensed length via N*(N-1)/2 == len.
    // `CondensedMatrix::from_vec(N, data)` accepts exactly N*(N-1)/2 values;
    // any other length is rejected. Verify for N = 2..10 and a few mismatches.
    for n in 2..=10 {
        let expected_len = n * (n - 1) / 2;
        let data = vec![1.0; expected_len];
        assert!(
            CondensedMatrix::from_vec(n, data).is_ok(),
            "n={} with len={} should succeed",
            n,
            expected_len
        );
    }

    // Mismatched lengths must error (scipy num_obs_y would raise on these).
    assert!(CondensedMatrix::from_vec(3, vec![1.0]).is_err());
    assert!(CondensedMatrix::from_vec(3, vec![1.0, 2.0]).is_err());
    assert!(CondensedMatrix::from_vec(3, vec![1.0, 2.0, 3.0, 4.0]).is_err());
    assert!(CondensedMatrix::from_vec(6, vec![1.0; 14]).is_err());
    assert!(CondensedMatrix::from_vec(6, vec![1.0; 16]).is_err());
}
