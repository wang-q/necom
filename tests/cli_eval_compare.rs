#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;
use std::io::Write;
use tempfile::Builder;

#[test]
fn command_nwk_compare_single_file() {
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
fn command_nwk_compare_single_file_one_tree_bails() {
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
fn command_nwk_compare_two_files() {
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
fn command_nwk_compare_branch_lengths() {
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
    // T1 vs T3: RF=2, WRF=0.8, KF=0.4

    // Check T1 vs T2
    // 1\t2\t0\t0.1\t0.1
    assert!(stdout.contains("1\t2\t0\t0.1\t0.1"));

    // Check T1 vs T3
    // 1\t3\t2\t0.8\t0.565685
    assert!(stdout.contains("1\t3\t2\t0.8\t0.565685"));
}

#[test]
fn command_nwk_compare_rejects_duplicate_leaf_names() {
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
