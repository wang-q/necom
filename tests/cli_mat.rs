#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;

/// Assert that a PHYLIP matrix row starting with `row_prefix` has value
/// approximately equal to `expected` at column index `col_idx`.
fn assert_row_value(
    stdout: &str,
    row_prefix: &str,
    col_idx: usize,
    expected: f32,
    tol: f32,
) {
    let line = stdout
        .lines()
        .find(|l| l.starts_with(row_prefix))
        .unwrap_or_else(|| panic!("missing row starting with '{}'", row_prefix));
    let parts: Vec<&str> = line.split('\t').collect();
    let value: f32 = parts[col_idx].parse().unwrap_or_else(|e| {
        panic!("failed to parse value at column {}: {}", col_idx, e)
    });
    assert!(
        (value - expected).abs() < tol,
        "{}: expected {} got {}",
        row_prefix,
        expected,
        value
    );
}

/// Assert that `mat compare` output contains `method` with approximate score.
fn assert_method_score(stdout: &str, method: &str, expected: f32, tol: f32) {
    let prefix = format!("{}\t", method);
    let line = stdout
        .lines()
        .find(|l| l.starts_with(&prefix))
        .unwrap_or_else(|| panic!("missing method '{}'", method));
    let score: f32 = line
        .split('\t')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap_or_else(|e| panic!("failed to parse score for {}: {}", method, e));
    assert!(
        (score - expected).abs() < tol,
        "{}: expected {} got {}",
        method,
        expected,
        score
    );
}

#[test]
fn command_mat_to_phylip() {
    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "to-phylip", "tests/mat/IBPA.fa.tsv"])
        .run();

    assert_eq!(stdout.lines().count(), 11);
    assert!(stdout.contains("IBPA_ECOLI\t0\t0.0669"));
}

#[test]
fn command_mat_to_pair() {
    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "to-pair", "tests/mat/IBPA.phy"])
        .run();

    assert_eq!(stdout.lines().count(), 55);
    assert!(stdout.contains("IBPA_ECOLI\tIBPA_ECOLI\t0\n"));
    assert!(stdout.contains("IBPA_ECOLI\tIBPA_ECOLI_GA\t0.058"));
}

#[test]
fn command_mat_format_full() {
    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "format", "tests/mat/IBPA.phy"])
        .run();

    assert_eq!(stdout.lines().count(), 11);
    assert!(stdout.contains("IBPA_ECOLI\t0\t0.058394\t0.160584"));
    assert!(stdout.contains("IBPA_ECOLI_GA\t0.058394\t0\t0.10219"));
}

#[test]
fn command_mat_format_lower() {
    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "format", "tests/mat/IBPA.phy", "--format", "lower"])
        .run();

    assert_eq!(stdout.lines().count(), 11);
    assert!(stdout.contains("IBPA_ECOLI\n"));
    assert!(stdout.contains("IBPA_ECOLI_GA\t0.058394\n"));
}

#[test]
fn command_mat_format_strict() {
    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "format", "tests/mat/IBPA.phy", "--format", "strict"])
        .run();

    assert_eq!(stdout.lines().count(), 11);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0].trim(), "10"); // Number of sequences line

    // Check format of the first sequence
    let first_seq = lines[1];
    assert!(first_seq.starts_with("IBPA_ECOLI"));
    assert_eq!(first_seq.chars().take(10).count(), 10); // Name length limit
    assert!(first_seq.contains(" 0.000000")); // Formatted distance value
}

#[test]
fn command_mat_subset() {
    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "subset", "tests/mat/IBPA.phy", "tests/mat/IBPA.list"])
        .run();

    // Verify output
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0].trim(), "3"); // Number of sequences
    assert!(lines[1].starts_with("IBPA_ECOLI_GA\t0.000000\t0.102190\t0.058394"));
    assert!(lines[3].starts_with("IBPA_ESCF3\t0.058394"));
}

#[test]
fn command_mat_subset_with_comments() {
    // ID list contains empty lines and comment lines, which should be ignored.
    let input = "IBPA_ECOLI_GA\n\n# comment\nIBPA_ECOLI_GA_LV\nIBPA_ESCF3\n";

    let (stdout, stderr) = NecomCmd::new()
        .args(&["mat", "subset", "tests/mat/IBPA.phy", "stdin"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0].trim(), "3"); // Number of sequences
    assert!(lines[1].starts_with("IBPA_ECOLI_GA\t0.000000\t0.102190\t0.058394"));
    assert!(!stderr.contains("Name not found: #"));
    assert!(!stderr.contains("Name not found: comment"));
}

#[test]
fn command_mat_compare() {
    // Test single method
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "compare",
            "tests/mat/IBPA.phy",
            "tests/mat/IBPA.71.phy",
            "--method",
            "pearson",
        ])
        .run();

    // Verify output format and approximate value
    assert!(stdout.contains("Method\tScore"));
    assert_method_score(&stdout, "pearson", 0.93, 0.01);

    // Test all methods
    let (stdout, stderr) = NecomCmd::new()
        .args(&[
            "mat",
            "compare",
            "tests/mat/IBPA.phy",
            "tests/mat/IBPA.71.phy",
            "--method",
            "all",
        ])
        .run();

    // Verify matrix information in stderr
    assert!(stderr.contains("Sequences in matrices: 10 and 10"));
    assert!(stderr.contains("Common sequences: 10"));

    // Verify all methods are present with approximate values
    assert_method_score(&stdout, "pearson", 0.93, 0.01);
    assert_method_score(&stdout, "spearman", 0.93, 0.01);
    assert_method_score(&stdout, "mae", 0.11, 0.01);
    assert_method_score(&stdout, "cosine", 0.97, 0.01);
    assert_method_score(&stdout, "jaccard", 0.75, 0.01);
    assert_method_score(&stdout, "euclid", 1.22, 0.01);
}

#[test]
fn command_mat_transform_linear() {
    // Input: A-B=0.1
    // Linear: x*2 + 1
    // Output: A-B=1.2
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "transform",
            "tests/mat/IBPA.phy",
            "--op",
            "linear",
            "--scale",
            "2.0",
            "--offset",
            "1.0",
        ])
        .run();

    // Original: IBPA_ECOLI vs IBPA_ECOLI_GA is 0.058394
    // Transformed: 0.058394 * 2 + 1 = 1.116788
    assert_row_value(&stdout, "IBPA_ECOLI\t", 2, 1.116788, 1e-6);
}

#[test]
fn command_mat_transform_inv_linear() {
    // Input: A-B=0.1
    // Inv-linear: 1.0 - x
    // Output: A-B=0.9
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "transform",
            "tests/mat/IBPA.phy",
            "--op",
            "inv-linear",
            "--max-val",
            "1.0",
        ])
        .run();

    // Original: IBPA_ECOLI vs IBPA_ECOLI_GA is 0.058394
    // Transformed: 1.0 - 0.058394 = 0.941606
    assert_row_value(&stdout, "IBPA_ECOLI\t", 2, 0.941606, 1e-6);
}

#[test]
fn command_mat_transform_log() {
    // Input: A-B=0.1
    // Log: -ln(x)
    // Output: -ln(0.1) = 2.302585
    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "transform", "tests/mat/IBPA.phy", "--op", "log"])
        .run();

    // Original: IBPA_ECOLI vs IBPA_ECOLI_GA is 0.058394
    // Transformed: -ln(0.058394) = 2.8405
    assert_row_value(&stdout, "IBPA_ECOLI\t", 2, 2.8405, 1e-4);
}

#[test]
fn command_mat_transform_normalize() {
    // Create a dummy matrix with non-zero diagonals for testing normalization
    // 3
    // A 1.0 0.5 0.5
    // B 0.5 4.0 1.0
    // C 0.5 1.0 9.0
    //
    // Norm(A,B) = 0.5 / sqrt(1*4) = 0.5/2 = 0.25
    // Norm(B,C) = 1.0 / sqrt(4*9) = 1.0/6 = 0.166667
    // Norm(A,C) = 0.5 / sqrt(1*9) = 0.5/3 = 0.166667

    let input = "3\nA\t1.0\t0.5\t0.5\nB\t0.5\t4.0\t1.0\nC\t0.5\t1.0\t9.0\n";

    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "transform", "stdin", "--normalize"])
        .stdin(input)
        .run();

    // Check normalized values
    assert_row_value(&stdout, "A\t", 2, 0.25, 1e-5); // A-B
    assert_row_value(&stdout, "A\t", 3, 0.166_667, 1e-5); // A-C
    assert_row_value(&stdout, "B\t", 3, 0.166_667, 1e-5); // B-C
    assert_row_value(&stdout, "A\t", 1, 1.0, 1e-6); // Diagonal
}

#[test]
fn command_mat_transform_normalize_inv() {
    // Combine normalize and inv-linear (Sim -> Dist)
    // Input same as above.
    // Norm(A,B) = 0.25
    // Inv: 1.0 - 0.25 = 0.75

    let input = "3\nA\t1.0\t0.5\t0.5\nB\t0.5\t4.0\t1.0\nC\t0.5\t1.0\t9.0\n";

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "transform",
            "stdin",
            "--normalize",
            "--op",
            "inv-linear",
        ])
        .stdin(input)
        .run();

    assert_row_value(&stdout, "A\t", 2, 0.75, 1e-5); // 1 - 0.25
    assert_row_value(&stdout, "A\t", 1, 0.0, 1e-6); // Diagonals: 1 - 1.0
}

#[test]
fn command_mat_transform_pairwise_stdin() {
    // Input: Pairwise TSV via STDIN
    // A B 0.1
    // A C 0.5
    // B C 0.2

    let input = "A\tB\t0.1\nA\tC\t0.5\nB\tC\t0.2\n";

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "transform",
            "stdin",
            "--input-format",
            "pair",
            "--op",
            "linear",
            "--scale",
            "2.0",
        ])
        .stdin(input)
        .run();

    // A-B=0.1 -> 0.2
    // A-C=0.5 -> 1.0
    // B-C=0.2 -> 0.4
    assert_row_value(&stdout, "A\t", 2, 0.2, 1e-6);
    assert_row_value(&stdout, "A\t", 3, 1.0, 1e-6);
    assert_row_value(&stdout, "B\t", 3, 0.4, 1e-6);
}

#[test]
fn command_mat_transform_pairwise_same_missing() {
    // Pairwise input without self-pairs and with a missing pair.
    let input = "A\tB\t0.1\nA\tC\t0.5\n";

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "transform",
            "stdin",
            "--input-format",
            "pair",
            "--same",
            "2.0",
            "--missing",
            "9.0",
        ])
        .stdin(input)
        .run();

    // Diagonals default to --same=2.0, missing pairs to --missing=9.0
    assert!(stdout.contains("2.000000"));
    assert!(stdout.contains("9.000000"));
}

#[test]
fn command_mat_transform_tsv_explicit() {
    // Should NOT auto-detect .tsv extension, must specify --input-format pair
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "transform",
            "tests/mat/IBPA.fa.tsv", // Using existing TSV file
            "--input-format",
            "pair",
        ])
        .run();

    // IBPA.fa.tsv contains: IBPA_ECOLI IBPA_ECOLI 0.0
    // IBPA_ECOLI IBPA_ECOLI_GA 0.0669
    // Default op is linear (x * 1 + 0), so values should be unchanged
    assert!(stdout.contains("0.0669"));
    assert!(stdout.lines().count() > 1); // Should output full matrix
}

#[test]
fn command_mat_compare_method_whitespace() {
    // Whitespace around comma-separated methods should be tolerated
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "compare",
            "tests/mat/IBPA.phy",
            "tests/mat/IBPA.71.phy",
            "--method",
            "pearson, cosine",
        ])
        .run();

    assert_method_score(&stdout, "pearson", 0.93, 0.01);
    assert_method_score(&stdout, "cosine", 0.97, 0.01);
}

#[test]
fn command_mat_to_phylip_malformed_warning() {
    // Malformed pairwise TSV lines should produce a warning but not fail
    let input = "A\tB\t0.1\nA\tB\nC\tD\t0.2\nE\tF\tnot-a-number\n";

    let (stdout, stderr) = NecomCmd::new()
        .args(&["mat", "to-phylip", "stdin"])
        .stdin(input)
        .run();

    assert!(stderr.contains("skipping malformed pairwise line"));
    assert!(stderr.contains("skipping pairwise line with invalid score"));
    assert!(stdout.contains("0.2"));
    assert_eq!(stdout.lines().count(), 5); // 4 valid sequences + header
}

#[test]
fn command_mat_format_lower_no_diag_input() {
    // Lower-triangular PHYLIP without diagonal values should be converted to full matrix.
    let input = "3\nA\nB 0.1\nC 0.2 0.3\n";

    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "format", "stdin"])
        .stdin(input)
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0].trim(), "3");
    assert!(lines[1].starts_with("A\t0\t0.1\t0.2"));
    assert!(lines[2].starts_with("B\t0.1\t0\t0.3"));
    assert!(lines[3].starts_with("C\t0.2\t0.3\t0"));
}

#[test]
fn command_mat_compare_empty_method_token() {
    // Empty tokens between commas should be skipped.
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "compare",
            "tests/mat/IBPA.phy",
            "tests/mat/IBPA.71.phy",
            "--method",
            "pearson,,cosine",
        ])
        .run();

    assert_method_score(&stdout, "pearson", 0.93, 0.01);
    assert_method_score(&stdout, "cosine", 0.97, 0.01);
    // Only two methods should be reported.
    assert_eq!(stdout.lines().count(), 3); // header + pearson + cosine
}

#[test]
fn command_mat_format_lower_extra_values_warning() {
    // Extra values in lower-triangular PHYLIP should produce a warning.
    let input = "3\nA\nB 0.1 0.999\nC 0.2 0.3\n";

    let (stdout, stderr) = NecomCmd::new()
        .args(&["mat", "format", "stdin"])
        .stdin(input)
        .run();

    assert!(stderr.contains("extra value(s)"));
    assert!(stderr.contains("LowerWithoutDiagonal"));

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0].trim(), "3");
    // The extra value 0.999 must be ignored.
    assert!(lines[2].starts_with("B\t0.1\t0\t0.3"));
}

#[test]
fn command_mat_compare_single_common_name() {
    // Two matrices sharing only 1 common name should bail with a clear error
    // rather than producing NaN metrics.
    use std::io::Write;
    let mut m1 = tempfile::NamedTempFile::new().unwrap();
    writeln!(m1, "2").unwrap();
    writeln!(m1, "A 0.0 0.1").unwrap();
    writeln!(m1, "B 0.1 0.0").unwrap();
    m1.flush().unwrap();

    let mut m2 = tempfile::NamedTempFile::new().unwrap();
    writeln!(m2, "2").unwrap();
    writeln!(m2, "A 0.0 0.2").unwrap();
    writeln!(m2, "C 0.2 0.0").unwrap();
    m2.flush().unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "mat",
            "compare",
            m1.path().to_str().unwrap(),
            m2.path().to_str().unwrap(),
            "--method",
            "pearson",
        ])
        .run_fail();

    assert!(
        stderr.contains("at least 2 common sequence names"),
        "expected degenerate-case error in stderr, got: {}",
        stderr
    );
}

#[test]
fn command_mat_compare_no_common_name() {
    // Two matrices with no common names should also bail.
    use std::io::Write;
    let mut m1 = tempfile::NamedTempFile::new().unwrap();
    writeln!(m1, "2").unwrap();
    writeln!(m1, "A 0.0 0.1").unwrap();
    writeln!(m1, "B 0.1 0.0").unwrap();
    m1.flush().unwrap();

    let mut m2 = tempfile::NamedTempFile::new().unwrap();
    writeln!(m2, "2").unwrap();
    writeln!(m2, "C 0.0 0.2").unwrap();
    writeln!(m2, "D 0.2 0.0").unwrap();
    m2.flush().unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "mat",
            "compare",
            m1.path().to_str().unwrap(),
            m2.path().to_str().unwrap(),
            "--method",
            "pearson",
        ])
        .run_fail();

    assert!(
        stderr.contains("at least 2 common sequence names"),
        "expected degenerate-case error in stderr, got: {}",
        stderr
    );
}

#[test]
fn command_mat_compare_nan_emits_na() {
    // Two matrices whose lower triangles are constant (zero variance).
    // pearson_correlation of two constant vectors is NaN; the output must
    // emit "NA" instead of "NaN" to keep the TSV parseable.
    use std::io::Write;
    let mut m1 = tempfile::NamedTempFile::new().unwrap();
    writeln!(m1, "3").unwrap();
    writeln!(m1, "A 0.0 0.5 0.5").unwrap();
    writeln!(m1, "B 0.5 0.0 0.5").unwrap();
    writeln!(m1, "C 0.5 0.5 0.0").unwrap();
    m1.flush().unwrap();

    let mut m2 = tempfile::NamedTempFile::new().unwrap();
    writeln!(m2, "3").unwrap();
    writeln!(m2, "A 0.0 0.9 0.9").unwrap();
    writeln!(m2, "B 0.9 0.0 0.9").unwrap();
    writeln!(m2, "C 0.9 0.9 0.0").unwrap();
    m2.flush().unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "compare",
            m1.path().to_str().unwrap(),
            m2.path().to_str().unwrap(),
            "--method",
            "pearson",
        ])
        .run();

    assert!(
        stdout.contains("pearson\tNA"),
        "expected 'pearson\\tNA' in stdout, got: {}",
        stdout
    );
}

#[test]
fn command_mat_to_phylip_empty_input() {
    // Empty pairwise input should produce a 0x0 PHYLIP matrix.
    let (stdout, stderr) = NecomCmd::new()
        .args(&["mat", "to-phylip", "stdin"])
        .stdin("")
        .run();

    assert_eq!(stdout, "0\n");
    assert!(stderr.is_empty(), "empty input should not produce warnings");
}

#[test]
fn command_mat_to_phylip_crlf() {
    // Pairwise TSV with CRLF line endings and extra whitespace should be parsed correctly.
    let input = "A\tB\t0.1\r\n  B\tC\t0.2 \r\nC\tA\t0.3\r\n";

    let (stdout, stderr) = NecomCmd::new()
        .args(&["mat", "to-phylip", "stdin"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A\t0\t0.1\t0.3"));
    assert!(stdout.contains("B\t0.1\t0\t0.2"));
    assert!(stdout.contains("C\t0.3\t0.2\t0"));
    assert!(!stderr.contains("skipping"));
}

#[test]
fn command_mat_to_phylip_empty_name() {
    // Lines with empty/whitespace names should be skipped without failing.
    let input = "\tB\t0.1\nA\t \t0.2\nA\tC\t0.3\n";

    let (stdout, stderr) = NecomCmd::new()
        .args(&["mat", "to-phylip", "stdin"])
        .stdin(input)
        .run();

    assert!(stderr.contains("skipping pairwise line with empty sequence name"));
    assert!(stdout.contains("A\t0\t0.3"));
    assert!(stdout.contains("C\t0.3\t0"));
}

#[test]
fn command_mat_compare_empty_method_errors() {
    // Empty --method should bail with a clear error instead of emitting only
    // the header line.
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "mat",
            "compare",
            "tests/mat/IBPA.phy",
            "tests/mat/IBPA.71.phy",
            "--method",
            "",
        ])
        .run_fail();

    assert!(
        stderr.contains("at least one comparison method required"),
        "expected empty-method error in stderr, got: {}",
        stderr
    );
}

#[test]
fn command_mat_subset_precision() {
    // Subset output should use fixed 6-decimal precision.
    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "subset", "tests/mat/IBPA.phy", "tests/mat/IBPA.list"])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0].trim(), "3");
    // The diagonal and first off-diagonal values should be emitted with
    // exactly 6 decimal places.
    assert!(lines[1].starts_with("IBPA_ECOLI_GA\t0.000000\t0.102190\t0.058394"));
}

#[test]
fn command_mat_transform_warns_same_missing_with_phylip() {
    // When --same / --missing are explicitly provided with default PHYLIP input,
    // a warning should be emitted because these flags only apply to --input-format pair.
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "mat",
            "transform",
            "tests/mat/IBPA.phy",
            "--same",
            "0.5",
            "--missing",
            "0.9",
        ])
        .run();

    assert!(
        stderr.contains("--same is ignored with --input-format phylip"),
        "expected --same warning in stderr, got: {}",
        stderr
    );
    assert!(
        stderr.contains("--missing is ignored with --input-format phylip"),
        "expected --missing warning in stderr, got: {}",
        stderr
    );
}

#[test]
fn command_mat_transform_no_warning_without_explicit_same_missing() {
    // Default --same / --missing values should NOT trigger warnings.
    let (_, stderr) = NecomCmd::new()
        .args(&["mat", "transform", "tests/mat/IBPA.phy"])
        .run();

    assert!(
        !stderr.contains("--same is ignored"),
        "unexpected --same warning without explicit flag: {}",
        stderr
    );
    assert!(
        !stderr.contains("--missing is ignored"),
        "unexpected --missing warning without explicit flag: {}",
        stderr
    );
}

#[test]
fn command_mat_transform_pair_with_same_missing_no_warning() {
    // With --input-format pair, --same / --missing are used and should NOT warn.
    let input = "A\tB\t0.1\n";
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "mat",
            "transform",
            "stdin",
            "--input-format",
            "pair",
            "--same",
            "0.5",
            "--missing",
            "0.9",
        ])
        .stdin(input)
        .run();

    assert!(
        !stderr.contains("--same is ignored"),
        "unexpected --same warning with pair input: {}",
        stderr
    );
    assert!(
        !stderr.contains("--missing is ignored"),
        "unexpected --missing warning with pair input: {}",
        stderr
    );
}

#[test]
fn command_mat_compare_reordered_names() {
    // Compare two matrices whose names appear in different orders. This
    // exercises the pre-computed index mapping in extract_common_lower_triangle.
    use std::io::Write;
    let mut m1 = tempfile::NamedTempFile::new().unwrap();
    writeln!(m1, "3").unwrap();
    writeln!(m1, "A 0.0 0.1 0.2").unwrap();
    writeln!(m1, "B 0.1 0.0 0.3").unwrap();
    writeln!(m1, "C 0.2 0.3 0.0").unwrap();
    m1.flush().unwrap();

    // m2 has the same values but names in reversed order (C, B, A).
    let mut m2 = tempfile::NamedTempFile::new().unwrap();
    writeln!(m2, "3").unwrap();
    writeln!(m2, "C 0.0 0.3 0.2").unwrap();
    writeln!(m2, "B 0.3 0.0 0.1").unwrap();
    writeln!(m2, "A 0.2 0.1 0.0").unwrap();
    m2.flush().unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "compare",
            m1.path().to_str().unwrap(),
            m2.path().to_str().unwrap(),
            "--method",
            "pearson",
        ])
        .run();

    // Identical lower triangles (after reordering) → perfect correlation.
    assert_method_score(&stdout, "pearson", 1.0, 1e-6);
}

#[test]
fn command_mat_format_numeric_name_with_values() {
    // A purely numeric sequence name on the first line is safe as long as the
    // line also contains distance values (so it cannot be a count header).
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "123 0.0 0.5").unwrap();
    writeln!(tmp, "456 0.5 0.0").unwrap();
    tmp.flush().unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "format", tmp.path().to_str().unwrap()])
        .run();

    assert!(stdout.contains("123\t0\t0.5"));
    assert!(stdout.contains("456\t0.5\t0"));
}

#[test]
fn command_mat_format_numeric_name_with_header() {
    // With an explicit count header, numeric sequence names can appear on any
    // data row, including the first one.
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "123 0.0 0.5").unwrap();
    writeln!(tmp, "456 0.5 0.0").unwrap();
    tmp.flush().unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&["mat", "format", tmp.path().to_str().unwrap()])
        .run();

    assert!(stdout.contains("123\t0\t0.5"));
    assert!(stdout.contains("456\t0.5\t0"));
}

#[test]
fn command_mat_format_numeric_header_mismatch_error() {
    // A single-integer first line is interpreted as a count header. If the
    // file does not contain that many rows, the error message should suggest
    // adding an explicit count header.
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "123").unwrap();
    writeln!(tmp, "456 0.1").unwrap();
    writeln!(tmp, "789 0.2 0.3").unwrap();
    tmp.flush().unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["mat", "format", tmp.path().to_str().unwrap()])
        .run_fail();

    assert!(
        stderr.contains("add an explicit count header"),
        "expected explicit-header hint in stderr, got: {}",
        stderr
    );
}
