#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;
use std::io::Write;
use tempfile::Builder;

#[test]
fn command_eval_compare_single_file() {
    // Create a temporary Newick file with 2 different trees
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    // Tree 1: ((A,B),(C,D)); -> Splits: {A,B} vs {C,D}
    // Tree 2: ((A,C),(B,D)); -> Splits: {A,C} vs {B,D}
    // RF distance should be 2.
    // Lengths are missing -> 0.0. WRF=0, KF=0.
    writeln!(file, "((A,B),(C,D));").unwrap();
    writeln!(file, "((A,C),(B,D));").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&["eval", "compare", file.path().to_str().unwrap()])
        .run();

    // Expected output (pairwise, no self-comparisons or duplicate pairs):
    // Tree1 Tree2 RF_Dist WRF_Dist KF_Dist
    // 1     2     2       0.000000 0.000000

    assert!(stdout.contains("Tree1\tTree2\tRF_Dist\tWRF_Dist\tKF_Dist"));
    assert!(stdout.contains("1\t2\t2\t0\t0"));
    // No self-comparisons or duplicate pairs (check line starts to avoid substring false positives)
    assert!(!stdout.lines().any(|l| l.starts_with("1\t1\t")));
    assert!(!stdout.lines().any(|l| l.starts_with("2\t1\t")));
    assert!(!stdout.lines().any(|l| l.starts_with("2\t2\t")));
}

#[test]
fn command_eval_compare_single_file_one_tree_bails() {
    // Single-file mode with <2 trees bails with a clear error rather than
    // warning and emitting an empty (header-only) table. This prevents users
    // from mistaking an empty result for a successful run.
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file, "((A,B),(C,D));").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["eval", "compare", file.path().to_str().unwrap()])
        .run_fail();

    assert!(
        stderr.contains("need at least 2 trees for pairwise comparison"),
        "expected bail message, got stderr: {}",
        stderr
    );
}

#[test]
fn command_eval_compare_single_file_empty_bails_with_no_trees_message() {
    // Single-file mode with an EMPTY file should report "no trees found in
    // first input file", not the misleading "need at least 2 trees" message.
    // Regression test for check ordering: the empty check must fire before
    // the `< 2` check, otherwise users see "got 0" instead of a clear
    // "no trees found" diagnostic.
    let empty_file = Builder::new().suffix(".nwk").tempfile().unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["eval", "compare", empty_file.path().to_str().unwrap()])
        .run_fail();

    assert!(
        stderr.contains("no trees found in first input file"),
        "expected 'no trees found' message for empty single-file input, got stderr: {}",
        stderr
    );
    assert!(
        !stderr.contains("need at least 2 trees"),
        "should not see 'need at least 2 trees' for an empty file, got stderr: {}",
        stderr
    );
}

#[test]
fn command_eval_compare_two_files() {
    let mut file1 = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file1, "((A,B),(C,D));").unwrap(); // Tree 1

    let mut file2 = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file2, "((A,B),(C,D));").unwrap(); // Tree 1 (Same)
    writeln!(file2, "((A,C),(B,D));").unwrap(); // Tree 2 (Diff, RF=2)

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "eval",
            "compare",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .run();

    // Expected:
    // T1(File1) vs T1(File2) -> 0
    // T1(File1) vs T2(File2) -> 2

    assert!(stdout.contains("1\t1\t0\t0\t0"));
    assert!(stdout.contains("1\t2\t2\t0\t0"));
}

#[test]
fn command_eval_compare_branch_lengths() {
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();

    // T1: Same topology, lengths 0.2
    writeln!(file, "((A:0.1,B:0.1):0.2,(C:0.1,D:0.1):0.2);").unwrap();

    // T2: Same topology, one length 0.3 (Diff 0.1)
    writeln!(file, "((A:0.1,B:0.1):0.3,(C:0.1,D:0.1):0.2);").unwrap();

    // T3: Diff topology, lengths 0.2
    writeln!(file, "((A:0.1,C:0.1):0.2,(B:0.1,D:0.1):0.2);").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&["eval", "compare", file.path().to_str().unwrap()])
        .run();

    // T1 vs T2: RF=0, WRF=0.1, KF=0.1
    // T1 vs T3: RF=2, WRF=0.8, KF=0.565685 (= sqrt(0.32))

    // Check T1 vs T2
    // 1\t2\t0\t0.1\t0.1
    assert!(stdout.contains("1\t2\t0\t0.1\t0.1"));

    // Check T1 vs T3
    // 1\t3\t2\t0.8\t0.565685
    assert!(stdout.contains("1\t3\t2\t0.8\t0.565685"));
}

#[test]
fn command_eval_compare_include_trivial() {
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    // Same topology, one branch length differs. With trivial splits included,
    // WRF/KF should account for the single-leaf branch length difference.
    writeln!(file, "((A:0.1,B:0.2):0.2,(C:0.1,D:0.1):0.2);").unwrap();
    writeln!(file, "((A:0.1,B:0.1):0.2,(C:0.1,D:0.1):0.2);").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "eval",
            "compare",
            file.path().to_str().unwrap(),
            "--include-trivial",
        ])
        .run();

    // Trivial split B has length 0.2 vs 0.1, contributing 0.1 to WRF and KF.
    assert!(stdout.contains("1\t2\t0\t0.1\t0.1"));
}

#[test]
fn command_eval_compare_rejects_duplicate_leaf_names() {
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    // Duplicate leaf name A should be rejected with a clear error.
    writeln!(file, "((A,A),(B,C));").unwrap();
    writeln!(file, "((A,B),(C,D));").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["eval", "compare", file.path().to_str().unwrap()])
        .run_fail();

    assert!(
        stderr.contains("duplicate leaf name"),
        "expected duplicate leaf name error, got stderr: {}",
        stderr
    );
}

#[test]
fn command_eval_compare_two_files_first_empty() {
    let empty_file = Builder::new().suffix(".nwk").tempfile().unwrap();
    let mut file2 = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file2, "((A,B),(C,D));").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "eval",
            "compare",
            empty_file.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .run_fail();

    assert!(
        stderr.contains("no trees found in first input file"),
        "expected empty first file error, got stderr: {}",
        stderr
    );
}

#[test]
fn command_eval_compare_two_files_second_empty() {
    let mut file1 = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file1, "((A,B),(C,D));").unwrap();
    let empty_file = Builder::new().suffix(".nwk").tempfile().unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "eval",
            "compare",
            file1.path().to_str().unwrap(),
            empty_file.path().to_str().unwrap(),
        ])
        .run_fail();

    assert!(
        stderr.contains("no trees found in second input file"),
        "expected empty second file error, got stderr: {}",
        stderr
    );
}

// ============================================================================
// Leaf-set mismatch regression tests
// ============================================================================
//
// `compute_tree_metrics` requires exact leaf-set equality between the two
// trees being compared. Previously, the writer was opened before any leaf-set
// validation, so a mismatch discovered mid-loop would leave a partial output
// file (header + some rows) on disk. The following tests verify that:
//
// 1. A leaf-set mismatch is detected and reported with a clear message.
// 2. The writer is NOT opened before validation, so an existing outfile is
//    not truncated when a mismatch is found.

#[test]
fn command_eval_compare_single_file_leaf_mismatch_bails() {
    // Tree 1 has leaves {A,B,C,D}; tree 2 has leaves {A,B,C,E}.
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file, "((A,B),(C,D));").unwrap();
    writeln!(file, "((A,B),(C,E));").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["eval", "compare", file.path().to_str().unwrap()])
        .run_fail();

    assert!(
        stderr.contains("different leaf set from tree 1"),
        "expected leaf-set mismatch error, got stderr: {}",
        stderr
    );
    assert!(
        stderr.contains("only here") && stderr.contains("only in first"),
        "expected diff details in error, got stderr: {}",
        stderr
    );
}

#[test]
fn command_eval_compare_two_files_cross_leaf_mismatch_bails() {
    // File 1 has {A,B,C,D}; file 2 has {A,B,C,E}.
    let mut file1 = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file1, "((A,B),(C,D));").unwrap();

    let mut file2 = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file2, "((A,B),(C,E));").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "eval",
            "compare",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ])
        .run_fail();

    assert!(
        stderr.contains("leaf sets differ between input files"),
        "expected cross-file leaf-set mismatch error, got stderr: {}",
        stderr
    );
}

#[test]
fn command_eval_compare_leaf_mismatch_does_not_truncate_outfile() {
    // Pre-populate an outfile with sentinel content. Run compare with inputs
    // that will fail leaf-set validation. The outfile must still contain the
    // sentinel content (i.e., the writer was never opened).
    let existing = sentinel_outfile("preserve me");
    let out_path = existing.path().to_str().unwrap();

    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file, "((A,B),(C,D));").unwrap();
    writeln!(file, "((A,B),(C,E));").unwrap();

    let (success, _, _) = run_with_outfile(
        &["eval", "compare", file.path().to_str().unwrap()],
        out_path,
    );
    assert!(!success, "expected failure for leaf-set mismatch");

    let preserved = std::fs::read_to_string(out_path).unwrap();
    assert_eq!(preserved, "preserve me");
}

#[test]
fn command_eval_compare_two_files_cross_mismatch_does_not_truncate_outfile() {
    // Same as above but for the two-file cross-file leaf-set mismatch path.
    let existing = sentinel_outfile("preserve me");
    let out_path = existing.path().to_str().unwrap();

    let mut file1 = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file1, "((A,B),(C,D));").unwrap();

    let mut file2 = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file2, "((A,B),(C,E));").unwrap();

    let (success, _, _) = run_with_outfile(
        &[
            "eval",
            "compare",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ],
        out_path,
    );
    assert!(
        !success,
        "expected failure for cross-file leaf-set mismatch"
    );

    let preserved = std::fs::read_to_string(out_path).unwrap();
    assert_eq!(preserved, "preserve me");
}

// Helpers for writer non-truncation regression tests.
fn run_with_outfile(args: &[&str], out_path: &str) -> (bool, String, String) {
    let mut cmd = assert_cmd::Command::cargo_bin("necom").unwrap();
    cmd.args(args);
    cmd.arg("--outfile").arg(out_path);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (output.status.success(), stdout, stderr)
}

fn sentinel_outfile(content: &str) -> tempfile::NamedTempFile {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(content.as_bytes()).unwrap();
    tmp
}
