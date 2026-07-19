mod common;
use crate::common::*;
use std::fs;

#[test]
fn test_scan_height() {
    // Tree: ((A:0.1,B:0.1):0.1,C:0.2);
    // Node heights (distance from leaves):
    // A, B, C: 0.0
    // (A,B): 0.1
    // Root ((A,B),C): 0.2
    let nwk = "((A:0.1,B:0.1):0.1,C:0.2);";
    let nwk_file = "tests/cut/scan_test.nwk";
    // Ensure dir exists
    if !std::path::Path::new("tests/cut").exists() {
        fs::create_dir_all("tests/cut").unwrap();
    }
    fs::write(nwk_file, nwk).expect("Failed to write nwk");

    let stats_file = "tests/cut/scan_stats.tsv";

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "cut",
            "scan-simple",
            nwk_file,
            "--height",
            "--range",
            "0,0.2,0.1",
            "--stats-out",
            stats_file,
        ])
        .run();

    // Verify stdout (Long Format)
    let out_lines: Vec<&str> = stdout.lines().collect();
    assert!(out_lines.len() > 1);
    assert_eq!(out_lines[0], "Group\tClusterID\tSampleID");

    // Verify stats file
    let stats_content =
        fs::read_to_string(stats_file).expect("Failed to read stats file");
    let lines: Vec<&str> = stats_content.lines().collect();

    // Header + 3 rows
    assert_eq!(lines.len(), 4, "Expected 4 lines output in stats file");
    assert_eq!(
        lines[0],
        "Group\tClusters\tSingletons\tNon-Singletons\tMaxSize"
    );

    // t=0
    let row0: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(row0[0], "height=0");
    assert_eq!(row0[1], "3"); // Clusters
    assert_eq!(row0[2], "3"); // Singletons
    assert_eq!(row0[3], "0"); // Non-Single
    assert_eq!(row0[4], "1"); // MaxSize

    // t=0.1
    let row1: Vec<&str> = lines[2].split('\t').collect();
    assert_eq!(row1[0], "height=0.1");
    assert_eq!(row1[1], "2");
    assert_eq!(row1[2], "1"); // {C}
    assert_eq!(row1[3], "1"); // {A,B}
    assert_eq!(row1[4], "2");

    // t=0.2
    let row2: Vec<&str> = lines[3].split('\t').collect();
    assert_eq!(row2[0], "height=0.2");
    assert_eq!(row2[1], "1");
    assert_eq!(row2[2], "0");
    assert_eq!(row2[3], "1"); // {A,B,C}
    assert_eq!(row2[4], "3");

    // Cleanup
    let _ = fs::remove_file(nwk_file);
    let _ = fs::remove_file(stats_file);
}

#[test]
fn test_scan_simple_bad_range_format() {
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "scan-simple",
            "tests/newick/abcde.nwk",
            "--height",
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
fn test_scan_simple_non_positive_step() {
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "scan-simple",
            "tests/newick/abcde.nwk",
            "--height",
            "--range",
            "0,1,0",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("step")
            && stderr.to_lowercase().contains("positive"),
        "Expected positive step error, got: {}",
        stderr
    );
}

#[test]
fn test_scan_k_integer_range() {
    let nwk = "((A:0.1,B:0.1):0.1,C:0.2);";
    let nwk_file = "tests/cut/scan_k_test.nwk";
    if !std::path::Path::new("tests/cut").exists() {
        fs::create_dir_all("tests/cut").unwrap();
    }
    fs::write(nwk_file, nwk).expect("Failed to write nwk");

    let (stdout, _) = NecomCmd::new()
        .args(&["cut", "scan-simple", nwk_file, "--k", "--range", "1,3,1"])
        .run();

    let out_lines: Vec<&str> = stdout.lines().collect();
    assert!(out_lines.len() > 1);
    assert_eq!(out_lines[0], "Group\tClusterID\tSampleID");
    assert!(out_lines.iter().any(|line| line.starts_with("k=1\t")));
    assert!(out_lines.iter().any(|line| line.starts_with("k=2\t")));
    assert!(out_lines.iter().any(|line| line.starts_with("k=3\t")));

    let _ = fs::remove_file(nwk_file);
}

#[test]
fn test_scan_k_range_rejects_float() {
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "scan-simple",
            "tests/newick/abcde.nwk",
            "--k",
            "--range",
            "1.5,3,1",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("integer")
            || stderr.to_lowercase().contains("invalid digit"),
        "Expected integer range error for k, got: {}",
        stderr
    );
}

#[test]
fn test_scan_k_range_rejects_zero_start() {
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "scan-simple",
            "tests/newick/abcde.nwk",
            "--k",
            "--range",
            "0,3,1",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("at least 1")
            || stderr.to_lowercase().contains("cluster count"),
        "Expected start >= 1 error for k scan, got: {}",
        stderr
    );
}

#[test]
fn test_scan_simple_missing_range() {
    // Without --range, clap must reject the invocation rather than panicking
    // on the later `args.get_one::<String>("range").unwrap()` call.
    let (_, stderr) = NecomCmd::new()
        .args(&["cut", "scan-simple", "tests/newick/abcde.nwk", "--height"])
        .run_fail();

    let lowered = stderr.to_lowercase();
    assert!(
        lowered.contains("--range") || lowered.contains("required"),
        "Expected missing --range error, got: {}",
        stderr
    );
}
