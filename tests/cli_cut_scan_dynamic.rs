mod common;
use crate::common::*;
use std::fs;

#[test]
fn test_scan_dynamic_tree() {
    // Tree with two well-separated pairs: ((A:0.1,B:0.1):0.8,(C:0.1,D:0.1):0.8);
    // min_module_size=2 -> {A,B},{C,D}
    // min_module_size=5 -> all unassigned (cluster 0)
    let nwk = "((A:0.1,B:0.1):0.8,(C:0.1,D:0.1):0.8);";
    let nwk_file = "tests/mat/scan_dyn_test.nwk";
    if !std::path::Path::new("tests/mat").exists() {
        fs::create_dir_all("tests/mat").unwrap();
    }
    fs::write(nwk_file, nwk).expect("Failed to write nwk");

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "cut",
            "scan-dynamic",
            nwk_file,
            "--range",
            "2,5,3", // val=2, then val=5
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() > 1);
    assert_eq!(lines[0], "Group\tClusterID\tSampleID");

    // Collect group labels
    let groups: Vec<&str> = lines[1..]
        .iter()
        .map(|l| l.split('\t').next().unwrap_or(""))
        .collect();
    let unique_groups: std::collections::HashSet<&str> = groups.into_iter().collect();
    // Must see two distinct scan groups (dynamic-tree=2 and dynamic-tree=5)
    assert_eq!(
        unique_groups.len(),
        2,
        "scan should produce 2 groups, got: {:?}",
        unique_groups
    );

    let _ = fs::remove_file(nwk_file);
}

#[test]
fn test_scan_dynamic_float_range_rejected() {
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "scan-dynamic",
            "tests/newick/abcde.nwk",
            "--range",
            "2.0,5.0,1.0",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("integer")
            || stderr.to_lowercase().contains("non-negative"),
        "Expected integer range error, got: {}",
        stderr
    );
}

#[test]
fn test_scan_dynamic_bad_range_format() {
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "scan-dynamic",
            "tests/newick/abcde.nwk",
            "--range",
            "0,1",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("format")
            && stderr.to_lowercase().contains("range"),
        "Expected bad range format error, got: {}",
        stderr
    );
}

#[test]
fn test_scan_dynamic_negative_range_rejected() {
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "scan-dynamic",
            "tests/newick/abcde.nwk",
            "--range=-1,2,1",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("non-negative"),
        "Expected non-negative range error, got: {}",
        stderr
    );
}
