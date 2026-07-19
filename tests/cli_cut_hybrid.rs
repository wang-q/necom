use std::fs;
use tempfile::Builder;

#[path = "common/mod.rs"]
mod common;
use common::NecomCmd;

#[test]
fn test_hybrid_cut_basic() {
    let temp = Builder::new()
        .prefix("necom_test_hybrid")
        .tempdir()
        .unwrap();
    let tree_file = temp.path().join("hybrid.nwk");
    let mat_file = temp.path().join("hybrid.phy");

    // Tree: ((A:0.1,B:0.1):0.8,(C:0.1,D:0.1):0.8);
    // Dynamic tree cut with min_size=2 should give 2 clusters: {A,B}, {C,D}.
    let tree_content = "((A:0.1,B:0.1):0.8,(C:0.1,D:0.1):0.8);";
    fs::write(&tree_file, tree_content).unwrap();

    // Matrix
    let mat_content = "4
A 0.0 0.2 1.0 1.0
B 0.2 0.0 1.0 1.0
C 1.0 1.0 0.0 0.2
D 1.0 1.0 0.2 0.0
";
    fs::write(&mat_file, mat_content).unwrap();

    let (stdout, _stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "hybrid",
            tree_file.to_str().unwrap(),
            "--matrix",
            mat_file.to_str().unwrap(),
            "--min-size",
            "2",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Should have 2 clusters: {A,B} and {C,D}
    let has_ab = lines.iter().any(|l| l.contains("A") && l.contains("B"));
    let has_cd = lines.iter().any(|l| l.contains("C") && l.contains("D"));

    assert!(has_ab, "Cluster {{A,B}} missing in output:\n{}", stdout);
    assert!(has_cd, "Cluster {{C,D}} missing in output:\n{}", stdout);
}

#[test]
fn test_hybrid_cut_pam() {
    let temp = Builder::new()
        .prefix("necom_test_hybrid_pam")
        .tempdir()
        .unwrap();
    let tree_file = temp.path().join("pam.nwk");
    let mat_file = temp.path().join("pam.phy");

    // Tree: ((A:0.1,B:0.1):0.8,(C:0.1,D:0.1):0.8,E:1.0);
    // min_size=2. {A,B}, {C,D}. E is singleton -> unassigned (Cluster 0).
    let tree_content = "((A:0.1,B:0.1):0.8,(C:0.1,D:0.1):0.8,E:1.0);";
    fs::write(&tree_file, tree_content).unwrap();

    // Matrix: E is closer to A/B (0.5) than C/D (1.0).
    let mat_content = "5
A 0.0 0.2 1.0 1.0 0.5
B 0.2 0.0 1.0 1.0 0.5
C 1.0 1.0 0.0 0.2 1.0
D 1.0 1.0 0.2 0.0 1.0
E 0.5 0.5 1.0 1.0 0.0
";
    fs::write(&mat_file, mat_content).unwrap();

    // 1. PAM is enabled by default.
    // In this case, E is a singleton (initially unassigned, cluster 0).
    // However, E is close to {A,B} (dist=0.5).
    // With PAM enabled, E should be reassigned to the {A,B} cluster.
    // We use --no-pam-dendro because in the tree, E is far from A/B (root split),
    // and standard PAM logic would prevent crossing such a high branch.

    let (stdout, _stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "hybrid",
            tree_file.to_str().unwrap(),
            "--matrix",
            mat_file.to_str().unwrap(),
            "--min-size",
            "2",
            "--no-pam-dendro", // Needed because E is far in tree
        ])
        .run();

    // Verify that E is grouped with A and B
    let lines: Vec<&str> = stdout.lines().collect();
    let has_abe = lines
        .iter()
        .any(|l| l.contains("A") && l.contains("B") && l.contains("E"));
    let has_cd = lines.iter().any(|l| l.contains("C") && l.contains("D"));

    assert!(
        has_abe,
        "Cluster {{A,B,E}} missing (PAM failed to reassign E):\n{}",
        stdout
    );
    assert!(has_cd, "Cluster {{C,D}} missing:\n{}", stdout);
}

#[test]
fn test_hybrid_min_size_zero_rejected() {
    let temp = Builder::new()
        .prefix("necom_test_hybrid_zero")
        .tempdir()
        .unwrap();
    let tree_file = temp.path().join("hybrid_zero.nwk");
    let mat_file = temp.path().join("hybrid_zero.phy");

    fs::write(&tree_file, "(A,B);").unwrap();
    fs::write(&mat_file, "2\nA 0.0 1.0\nB 1.0 0.0\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "hybrid",
            tree_file.to_str().unwrap(),
            "--matrix",
            mat_file.to_str().unwrap(),
            "--min-size",
            "0",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("min cluster size")
            && stderr.to_lowercase().contains("greater than 0"),
        "Expected min-size >0 error, got: {}",
        stderr
    );
}

#[test]
fn test_hybrid_missing_min_size() {
    let temp = Builder::new()
        .prefix("necom_test_hybrid_missing")
        .tempdir()
        .unwrap();
    let tree_file = temp.path().join("hybrid_missing.nwk");
    let mat_file = temp.path().join("hybrid_missing.phy");

    fs::write(&tree_file, "(A,B);").unwrap();
    fs::write(&mat_file, "2\nA 0.0 1.0\nB 1.0 0.0\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "hybrid",
            tree_file.to_str().unwrap(),
            "--matrix",
            mat_file.to_str().unwrap(),
        ])
        .run_fail();

    let lowered = stderr.to_lowercase();
    assert!(
        lowered.contains("--min-size") || lowered.contains("required"),
        "Expected missing --min-size error, got: {}",
        stderr
    );
}
