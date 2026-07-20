use std::collections::{HashMap, HashSet};
use std::fs;
use tempfile::Builder;

#[path = "common/mod.rs"]
mod common;
use common::NecomCmd;

fn parse_clusters(output: &str) -> Vec<HashSet<String>> {
    let mut clusters = Vec::new();
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let set: HashSet<String> =
            line.split_whitespace().map(|s| s.to_string()).collect();
        clusters.push(set);
    }
    clusters.sort_by(|a, b| {
        let min_a = a.iter().min().unwrap();
        let min_b = b.iter().min().unwrap();
        min_a.cmp(min_b)
    });
    clusters
}

#[test]
fn test_dynamic_tree_cut_basic() {
    let temp = Builder::new()
        .prefix("necom_test_dynamic")
        .tempdir()
        .unwrap();
    let tree_file = temp.path().join("basic.nwk");
    let tree_content = "((A:0.1,B:0.1):0.8,(C:0.1,D:0.1):0.8);";
    fs::write(&tree_file, tree_content).unwrap();

    let (stdout, _stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "dynamic",
            tree_file.to_str().unwrap(),
            "--min-size",
            "2",
        ])
        .run();

    let clusters = parse_clusters(&stdout);
    // Should have 2 clusters: {A,B} and {C,D}
    assert_eq!(
        clusters.len(),
        2,
        "expected 2 clusters, got {}:\n{}",
        clusters.len(),
        stdout
    );
    let expected_ab: HashSet<String> =
        ["A", "B"].iter().map(|s| s.to_string()).collect();
    let expected_cd: HashSet<String> =
        ["C", "D"].iter().map(|s| s.to_string()).collect();
    assert!(
        clusters.contains(&expected_ab),
        "Cluster {{A,B}} missing in output:\n{}",
        stdout
    );
    assert!(
        clusters.contains(&expected_cd),
        "Cluster {{C,D}} missing in output:\n{}",
        stdout
    );
}

#[test]
fn test_dynamic_tree_cut_unassigned() {
    let temp = Builder::new()
        .prefix("necom_test_dynamic_un")
        .tempdir()
        .unwrap();
    let tree_file = temp.path().join("unassigned.nwk");

    // Tree where min size is too large for leaves
    let tree_content = "((A:0.1,B:0.1):0.8,(C:0.1,D:0.1):0.8);";
    fs::write(&tree_file, tree_content).unwrap();

    // Min size 5. Total leaves 4.
    // Should result in unassigned (Cluster 0) or empty output if 0 is suppressed?
    // Our implementation currently outputs all clusters in the map.
    // Dynamic tree assigns 0 to unassigned nodes.
    // Partition.get_clusters() groups by value.
    // So we expect a cluster with ID 0 containing A,B,C,D (or however many are unassigned).
    // Or maybe multiple unassigned clusters? No, 0 is a single ID.

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "cut",
            "dynamic",
            tree_file.to_str().unwrap(),
            "--min-size",
            "5",
            "--format",
            "pair", // Easier to check IDs
        ])
        .run();

    // In pair format: Rep\tMember
    // If cluster ID is 0, it will be treated as a valid cluster by the writer code.
    // So we should see output.

    assert!(!stdout.is_empty());
    // Since all are unassigned (ID 0), they form one "cluster".
    // So we expect 4 lines, all belonging to the same representative.
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4);
}

#[test]
fn test_dynamic_tree_cut_multi_level() {
    let temp = Builder::new()
        .prefix("necom_test_dynamic_multi")
        .tempdir()
        .unwrap();
    let tree_file = temp.path().join("multi.nwk");
    // Heights:
    //   (A,B) = 10.0
    //   (C,D) = 0.1
    //   quartet = 10.1
    //   E = 0.0
    //   root = 10.2
    // cut_height = 0.99 * 10.2 = 10.098
    // root (10.2) > cut_height -> descend
    // quartet (10.1) > cut_height -> descend
    // (A,B) (10.0) <= cut_height -> cluster root {A,B}
    // (C,D) (0.1) <= cut_height -> cluster root {C,D}
    // This verifies that cutree_static can perform multi-level cuts now that
    // all node heights are computed, not just the root.
    let tree_content = "(((A:10,B:10):0.1,(C:0.1,D:0.1):0.1):0.1,E:0.1);";
    fs::write(&tree_file, tree_content).unwrap();

    let (stdout, _stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "dynamic",
            tree_file.to_str().unwrap(),
            "--min-size",
            "2",
        ])
        .run();

    let clusters = parse_clusters(&stdout);
    let expected: Vec<HashSet<String>> = vec![
        ["A", "B"].iter().map(|s| s.to_string()).collect(),
        ["C", "D"].iter().map(|s| s.to_string()).collect(),
        ["E"].iter().map(|s| s.to_string()).collect(),
    ];
    assert_eq!(clusters, expected);
}

#[test]
fn test_dynamic_tree_cut_with_support_filter() {
    let temp = Builder::new()
        .prefix("necom_test_dynamic_support")
        .tempdir()
        .unwrap();
    let tree_file = temp.path().join("support.nwk");
    // (C,D) internal node has support 50.
    // Heights without support filter:
    //   (A,B) = 0.1, (C,D) = 0.1, quartet = 0.2, E = 0.0, root = 1.0
    //   cut_height = 0.99 -> quartet stays whole -> {A,B,C,D}, {E}
    // With --support 60:
    //   edge (C,D)->parent becomes 1e20
    //   (C,D) height = 1e20, quartet height = 1e20, root height = 1e20
    //   cut_height = 0.99 * 1e20
    //   root and quartet exceed cut_height, so cutree_static descends
    //   (A,B) height 0.2 <= cut_height -> cluster {A,B}
    //   C and D fall below min_module_size 2 -> unassigned (cluster 0)
    //   E also unassigned (cluster 0)
    let tree_content = "(((A:0.1,B:0.1):0.1,(C:0.1,D:0.1)50:0.1):0.8,E:0.1);";
    fs::write(&tree_file, tree_content).unwrap();

    // Without support: one cluster of four leaves + E
    let (out_no_support, _) = NecomCmd::new()
        .args(&[
            "cut",
            "dynamic",
            tree_file.to_str().unwrap(),
            "--min-size",
            "2",
        ])
        .run();
    let clusters_no_support = parse_clusters(&out_no_support);
    assert!(
        clusters_no_support.iter().any(|c| {
            c.len() == 4
                && c.contains("A")
                && c.contains("B")
                && c.contains("C")
                && c.contains("D")
        }),
        "expected {{A,B,C,D}} cluster without support, got {:?}",
        clusters_no_support
    );
    assert!(
        clusters_no_support
            .iter()
            .any(|c| c == &["E"].iter().map(|s| s.to_string()).collect()),
        "expected {{E}} cluster without support"
    );

    // With support 60: masked edge forces deeper cut, producing {A,B}; C, D, E unassigned
    let (out_support, _) = NecomCmd::new()
        .args(&[
            "cut",
            "dynamic",
            tree_file.to_str().unwrap(),
            "--min-size",
            "2",
            "--support",
            "60",
        ])
        .run();
    let clusters_support = parse_clusters(&out_support);
    assert!(
        clusters_support
            .iter()
            .any(|c| c == &["A", "B"].iter().map(|s| s.to_string()).collect()),
        "expected {{A,B}} cluster with support, got {:?}",
        clusters_support
    );
    assert!(
        !clusters_support.iter().any(|c| c.len() == 4),
        "support filter should break the quartet apart"
    );
}

#[test]
fn test_dynamic_tree_cut_asymmetric_heights() {
    let temp = Builder::new()
        .prefix("necom_test_dynamic_asym")
        .tempdir()
        .unwrap();
    let tree_file = temp.path().join("asym.nwk");

    // Left clade (A,B) has height 10; right clade (C,D) has height 0.1.
    // cut_height = 0.99 * 10.1 ≈ 9.999.
    // Root and (A,B) exceed cut_height, so they are split; A and B fall
    // below min_module_size 2 and become unassigned. Each unassigned leaf is
    // emitted as its own singleton cluster. (C,D) stays a cluster.
    let tree_content = "((A:10,B:0.1):0.1,(C:0.1,D:0.1):0.1);";
    fs::write(&tree_file, tree_content).unwrap();

    let (stdout, _stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "dynamic",
            tree_file.to_str().unwrap(),
            "--min-size",
            "2",
            "--format",
            "pair",
        ])
        .run();

    // Group members by their cluster representative.
    let mut rep_to_members: HashMap<String, HashSet<String>> = HashMap::new();
    for line in stdout.lines() {
        let mut parts = line.split_whitespace();
        let rep = parts.next().unwrap().to_string();
        let member = parts.next().unwrap().to_string();
        rep_to_members.entry(rep).or_default().insert(member);
    }

    let mut clusters: Vec<HashSet<String>> = rep_to_members.into_values().collect();
    clusters.sort_by(|a, b| {
        let min_a = a.iter().min().unwrap();
        let min_b = b.iter().min().unwrap();
        min_a.cmp(min_b)
    });

    let expected: Vec<HashSet<String>> = vec![
        ["A"].iter().map(|s| s.to_string()).collect(),
        ["B"].iter().map(|s| s.to_string()).collect(),
        ["C", "D"].iter().map(|s| s.to_string()).collect(),
    ];
    assert_eq!(clusters, expected);
}

#[test]
fn test_dynamic_min_size_zero_rejected() {
    let temp = Builder::new()
        .prefix("necom_test_dynamic_zero")
        .tempdir()
        .unwrap();
    let tree_file = temp.path().join("zero.nwk");
    fs::write(&tree_file, "(A,B);").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "dynamic",
            tree_file.to_str().unwrap(),
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
fn test_dynamic_missing_min_size() {
    let temp = Builder::new()
        .prefix("necom_test_dynamic_missing")
        .tempdir()
        .unwrap();
    let tree_file = temp.path().join("missing.nwk");
    fs::write(&tree_file, "(A,B);").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["cut", "dynamic", tree_file.to_str().unwrap()])
        .run_fail();

    let lowered = stderr.to_lowercase();
    assert!(
        lowered.contains("--min-size") || lowered.contains("required"),
        "Expected missing --min-size error, got: {}",
        stderr
    );
}
