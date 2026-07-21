#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;

#[test]
fn command_clust_cc() {
    let (stdout, _) = NecomCmd::new()
        .args(&["clust", "cc", "tests/clust/IBPA.fa.05.tsv"])
        .run();

    assert_eq!(stdout.lines().count(), 7);
    assert!(
        stdout.contains("A0A192CFC5_ECO25\tIBPA_ECOLI\tIBPA_ESCF3\nIBPA_ECOLI_GA_LV")
    );
}

#[test]
fn command_clust_cc_pair() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "cc",
            "tests/clust/IBPA.fa.05.tsv",
            "--format",
            "pair",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 10);
    assert!(stdout.contains("A0A192CFC5_ECO25\tIBPA_ECOLI"));
    assert!(stdout.contains("IBPA_ECOLI_GA_LV\tIBPA_ECOLI_GA_LV"));
}

#[test]
fn command_clust_dbscan() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            "tests/clust/IBPA.fa.tsv",
            "--eps",
            "0.05",
            "--min-points",
            "2",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 7);
    // Default representative is medoid; the first column for the {A0A, IBPA_ECOLI, IBPA_ESCF3}
    // cluster should be IBPA_ECOLI (minimum sum of distances; tie broken alphabetically).
    assert!(stdout.contains("IBPA_ECOLI\tA0A192CFC5_ECO25\tIBPA_ESCF3"));
}

#[test]
fn command_clust_dbscan_rep_first() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            "tests/clust/IBPA.fa.tsv",
            "--eps",
            "0.05",
            "--min-points",
            "2",
            "--rep",
            "first",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 7);
    assert!(stdout.contains("A0A192CFC5_ECO25\tIBPA_ECOLI\tIBPA_ESCF3"));
}

#[test]
fn command_clust_dbscan_pair() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            "tests/clust/IBPA.fa.tsv",
            "--eps",
            "0.05",
            "--min-points",
            "2",
            "--format",
            "pair",
        ])
        .run();

    // Each line contains a representative-member pair
    assert!(stdout.lines().count() > 0);
    assert!(
        stdout.contains("IBPA_ECOLI\tIBPA_ECOLI\n")
            || stdout.contains("IBPA_ESCF3\tIBPA_ESCF3\n")
    );
    assert!(
        stdout.contains("IBPA_ECOLI\tIBPA_ESCF3\n")
            || stdout.contains("IBPA_ESCF3\tIBPA_ECOLI\n")
    );
}

#[test]
fn command_clust_kmedoids() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "km",
            "tests/clust/IBPA.fa.tsv",
            "-k",
            "2",
            "--seed",
            "42",
        ])
        .run();

    // Output should contain at least 2 lines (clusters)
    assert!(stdout.lines().count() >= 2);
}

#[test]
fn command_clust_kmedoids_pair() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "k-medoids",
            "tests/clust/IBPA.fa.tsv",
            "-k",
            "2",
            "--format",
            "pair",
            "--seed",
            "42",
        ])
        .run();

    // Should contain tab-separated pairs
    assert!(stdout.contains("\t"));
    assert!(stdout.lines().count() >= 2);
}

#[test]
fn command_clust_mcl() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "mcl",
            "tests/clust/mcl_test.tsv",
            "--inflation",
            "2.0",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("A\tB\tC"));
    assert!(stdout.contains("D\tE"));
}

#[test]
fn command_clust_mcl_complex() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "mcl",
            "tests/clust/mcl_complex.tsv",
            "--inflation",
            "2.0",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 2);
    // Cluster 1: n1, n2, n3, n4
    assert!(stdout.contains("n1\tn2\tn3\tn4"));
    // Cluster 2: n5, n6, n7
    assert!(stdout.contains("n5\tn6\tn7"));
}

#[test]
fn command_clust_mcl_args() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "mcl",
            "tests/clust/mcl_test.tsv",
            "--prune",
            "1e-3",
            "--max-iter",
            "50",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("A\tB\tC"));
    assert!(stdout.contains("D\tE"));
}

#[test]
fn command_clust_mcl_pair() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "mcl",
            "tests/clust/mcl_test.tsv",
            "--format",
            "pair",
        ])
        .run();

    // Cluster 1 (size 3) + Cluster 2 (size 2) = 5 pairs
    assert_eq!(stdout.lines().count(), 5);

    // Check representative output
    assert!(stdout.contains("A\tA"));
    assert!(stdout.contains("A\tB"));
    assert!(stdout.contains("A\tC"));
    assert!(stdout.contains("D\tD"));
    assert!(stdout.contains("D\tE"));
}

#[test]
fn command_clust_dbscan_default_min_points() {
    let (stdout, _) = NecomCmd::new().args(&["clust", "dbscan", "--help"]).run();

    // The help should show default value of 4 for --min-points.
    let min_points_line = stdout
        .lines()
        .find(|l| l.contains("--min-points"))
        .expect("--min-points line not found in help");
    assert!(
        min_points_line.contains("[default: 4]"),
        "expected default 4 in --min-points help, got: {}",
        min_points_line
    );
}

#[test]
fn command_clust_dbscan_pair_rep_first() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(
        &input,
        "A\tB\t0.1\nA\tC\t0.2\nB\tC\t0.15\nA\tD\t0.5\nB\tD\t0.6\nC\tD\t0.55\n",
    )
    .unwrap();

    let (stdout_medoid_pair, _) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            input.to_str().unwrap(),
            "--eps",
            "0.25",
            "--min-points",
            "2",
            "--format",
            "pair",
        ])
        .run();

    let (stdout_first_pair, _) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            input.to_str().unwrap(),
            "--eps",
            "0.25",
            "--min-points",
            "2",
            "--format",
            "pair",
            "--rep",
            "first",
        ])
        .run();

    let (stdout_medoid_cluster, _) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            input.to_str().unwrap(),
            "--eps",
            "0.25",
            "--min-points",
            "2",
            "--format",
            "cluster",
        ])
        .run();

    let (stdout_first_cluster, _) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            input.to_str().unwrap(),
            "--eps",
            "0.25",
            "--min-points",
            "2",
            "--format",
            "cluster",
            "--rep",
            "first",
        ])
        .run();

    // Default medoid representative is B (min sum of distances).
    assert!(stdout_medoid_pair.contains("B\tA"));
    assert!(stdout_medoid_pair.contains("B\tB"));
    assert!(stdout_medoid_pair.contains("B\tC"));
    let cluster_medoid_line = stdout_medoid_cluster.lines().next().unwrap();
    assert!(cluster_medoid_line.starts_with("B\t"));

    // With --rep first, representative is A (alphabetically first).
    assert!(stdout_first_pair.contains("A\tA"));
    assert!(stdout_first_pair.contains("A\tB"));
    assert!(stdout_first_pair.contains("A\tC"));
    let cluster_first_line = stdout_first_cluster.lines().next().unwrap();
    assert!(cluster_first_line.starts_with("A\t"));
}

#[test]
fn command_clust_dbscan_min_pct() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(
        &input,
        "A\tB\t0.1\nA\tC\t0.2\nB\tC\t0.15\nA\tD\t0.5\nB\tD\t0.6\nC\tD\t0.55\n",
    )
    .unwrap();

    // 4 samples; 0.5 -> ceil(2.0) = 2, should match --min-points 2.
    let (stdout_pct, _) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            input.to_str().unwrap(),
            "--eps",
            "0.25",
            "--min-pct",
            "0.5",
            "--format",
            "cluster",
        ])
        .run();

    let (stdout_points, _) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            input.to_str().unwrap(),
            "--eps",
            "0.25",
            "--min-points",
            "2",
            "--format",
            "cluster",
        ])
        .run();

    assert_eq!(stdout_pct, stdout_points);
}

#[test]
fn command_clust_dbscan_min_pct_conflicts_with_min_points() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\nA\tC\t0.2\nB\tC\t0.15\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            input.to_str().unwrap(),
            "--eps",
            "0.25",
            "--min-points",
            "2",
            "--min-pct",
            "0.5",
        ])
        .run_fail();

    assert!(
        stderr.contains("mutually exclusive")
            || stderr.contains("--min-points")
            || stderr.contains("--min-pct"),
        "expected mutual exclusion error, got stderr: {}",
        stderr
    );
}

#[test]
fn command_clust_mcl_pair_rep_first() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t1.0\nB\tC\t1.0\nC\tA\t0.5\nD\tE\t1.0\n").unwrap();

    let (stdout_medoid, _) = NecomCmd::new()
        .args(&["clust", "mcl", input.to_str().unwrap(), "--format", "pair"])
        .run();

    let (stdout_first, _) = NecomCmd::new()
        .args(&[
            "clust",
            "mcl",
            input.to_str().unwrap(),
            "--format",
            "pair",
            "--rep",
            "first",
        ])
        .run();

    let (stdout_medoid_cluster, _) = NecomCmd::new()
        .args(&[
            "clust",
            "mcl",
            input.to_str().unwrap(),
            "--format",
            "cluster",
        ])
        .run();

    let (stdout_first_cluster, _) = NecomCmd::new()
        .args(&[
            "clust",
            "mcl",
            input.to_str().unwrap(),
            "--format",
            "cluster",
            "--rep",
            "first",
        ])
        .run();

    // Default medoid (max similarity sum) representative is B.
    assert!(stdout_medoid.contains("B\tA"));
    assert!(stdout_medoid.contains("B\tB"));
    assert!(stdout_medoid.contains("B\tC"));
    let cluster_medoid_line = stdout_medoid_cluster.lines().next().unwrap();
    assert!(cluster_medoid_line.starts_with("B\t"));

    // With --rep first, representative is A.
    assert!(stdout_first.contains("A\tA"));
    assert!(stdout_first.contains("A\tB"));
    assert!(stdout_first.contains("A\tC"));
    let cluster_first_line = stdout_first_cluster.lines().next().unwrap();
    assert!(cluster_first_line.starts_with("A\t"));
}

#[test]
fn command_clust_kmedoids_pair_rep_first() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(
        &input,
        "A\tB\t0.1\nA\tC\t0.2\nB\tC\t0.15\nA\tD\t0.5\nB\tD\t0.6\nC\tD\t0.55\n",
    )
    .unwrap();

    let (stdout_medoid, _) = NecomCmd::new()
        .args(&[
            "clust",
            "k-medoids",
            input.to_str().unwrap(),
            "-k",
            "2",
            "--format",
            "pair",
            "--seed",
            "42",
        ])
        .run();

    let (stdout_first, _) = NecomCmd::new()
        .args(&[
            "clust",
            "k-medoids",
            input.to_str().unwrap(),
            "-k",
            "2",
            "--format",
            "pair",
            "--rep",
            "first",
            "--seed",
            "42",
        ])
        .run();

    let (stdout_medoid_cluster, _) = NecomCmd::new()
        .args(&[
            "clust",
            "k-medoids",
            input.to_str().unwrap(),
            "-k",
            "2",
            "--format",
            "cluster",
            "--seed",
            "42",
        ])
        .run();

    let (stdout_first_cluster, _) = NecomCmd::new()
        .args(&[
            "clust",
            "k-medoids",
            input.to_str().unwrap(),
            "-k",
            "2",
            "--format",
            "cluster",
            "--rep",
            "first",
            "--seed",
            "42",
        ])
        .run();

    // Verify both produce valid pair output.
    assert!(stdout_medoid.contains("\t"));
    assert!(stdout_first.contains("\t"));
    assert!(stdout_medoid.lines().count() >= 2);
    assert!(stdout_first.lines().count() >= 2);

    // In cluster format, representative is placed in the first column.
    let cluster_medoid_first_line = stdout_medoid_cluster.lines().next().unwrap();
    let cluster_first_first_line = stdout_first_cluster.lines().next().unwrap();
    assert!(cluster_medoid_first_line.starts_with("B\t"));
    assert!(cluster_first_first_line.starts_with("A\t"));
}

#[test]
fn command_clust_dbscan_invalid_eps() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["clust", "dbscan", input.to_str().unwrap(), "--eps", "0"])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("eps"),
        "expected eps error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_dbscan_invalid_min_points() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            input.to_str().unwrap(),
            "--min-points",
            "0",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("min-points"),
        "expected min-points error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_kmedoids_invalid_k() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["clust", "k-medoids", input.to_str().unwrap(), "-k", "0"])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("--k"),
        "expected --k error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_mcl_invalid_inflation() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t1.0\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "clust",
            "mcl",
            input.to_str().unwrap(),
            "--inflation",
            "1.0",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("inflation"),
        "expected inflation error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_mcl_invalid_max_iter() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t1.0\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["clust", "mcl", input.to_str().unwrap(), "--max-iter", "0"])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("max-iter"),
        "expected max-iter error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_kmedoids_invalid_k_too_large() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["clust", "k-medoids", input.to_str().unwrap(), "-k", "10"])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("samples"),
        "expected samples error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_cc_empty() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("empty.tsv");
    std::fs::write(&input, "").unwrap();

    let (stdout, stderr) = NecomCmd::new()
        .args(&["clust", "cc", input.to_str().unwrap()])
        .run();

    assert!(stderr.is_empty(), "expected no stderr, got: {}", stderr);
    assert!(
        stdout.trim().is_empty(),
        "expected empty output, got: {}",
        stdout
    );
}

#[test]
fn command_clust_dbscan_eps_nan() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["clust", "dbscan", input.to_str().unwrap(), "--eps", "nan"])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("eps"),
        "expected eps error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_dbscan_eps_inf() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["clust", "dbscan", input.to_str().unwrap(), "--eps", "inf"])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("eps"),
        "expected eps error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_mcl_prune_negative() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t1.0\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["clust", "mcl", input.to_str().unwrap(), "--prune=-1"])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("prune"),
        "expected prune error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_mcl_prune_nan() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t1.0\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["clust", "mcl", input.to_str().unwrap(), "--prune", "nan"])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("prune"),
        "expected prune error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_kmedoids_runs_zero() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "clust",
            "k-medoids",
            input.to_str().unwrap(),
            "-k",
            "2",
            "--runs",
            "0",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("runs"),
        "expected runs error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_kmedoids_max_iter_zero() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "clust",
            "k-medoids",
            input.to_str().unwrap(),
            "-k",
            "2",
            "--max-iter",
            "0",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("max-iter"),
        "expected max-iter error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_dbscan_same_nan() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["clust", "dbscan", input.to_str().unwrap(), "--same", "nan"])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("same"),
        "expected --same error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_dbscan_missing_nan() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "clust",
            "dbscan",
            input.to_str().unwrap(),
            "--missing",
            "nan",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("missing"),
        "expected --missing error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_kmedoids_same_nan() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "clust",
            "k-medoids",
            input.to_str().unwrap(),
            "-k",
            "2",
            "--same",
            "nan",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("same"),
        "expected --same error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_kmedoids_missing_nan() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "clust",
            "k-medoids",
            input.to_str().unwrap(),
            "-k",
            "2",
            "--missing",
            "nan",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("missing"),
        "expected --missing error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_mcl_same_nan() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t1.0\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["clust", "mcl", input.to_str().unwrap(), "--same", "nan"])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("same"),
        "expected --same error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_mcl_missing_nan() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t1.0\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["clust", "mcl", input.to_str().unwrap(), "--missing", "nan"])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("missing"),
        "expected --missing error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_mcl_inflation_nan() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t1.0\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "clust",
            "mcl",
            input.to_str().unwrap(),
            "--inflation",
            "nan",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("inflation"),
        "expected inflation error, got: {}",
        stderr
    );
}

#[test]
fn command_clust_mcl_inflation_inf() {
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t1.0\n").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "clust",
            "mcl",
            input.to_str().unwrap(),
            "--inflation",
            "inf",
        ])
        .run_fail();

    assert!(
        stderr.to_lowercase().contains("inflation"),
        "expected inflation error, got: {}",
        stderr
    );
}

// ============================================================================
// Writer non-truncation regression tests
// ============================================================================
//
// Each clust subcommand must open its writer only after all input loading and
// validation has succeeded. If the writer were opened first, an input-loading
// failure would truncate an existing outfile (because `File::create` is used),
// surprising the user. The following tests pre-populate an outfile with
// sentinel content, then run each subcommand with a bad input pointing at
// that outfile, and verify that the command failed AND the outfile still
// contains the sentinel content (was not truncated).

/// Run `necom` with the given subcommand args plus `--outfile <out_path>` and
/// return `(success, stdout, stderr)`.
fn run_with_outfile(args: &[&str], out_path: &str) -> (bool, String, String) {
    let mut cmd = assert_cmd::Command::cargo_bin("necom").unwrap();
    cmd.args(args);
    cmd.arg("--outfile").arg(out_path);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (output.status.success(), stdout, stderr)
}

/// Pre-create a named temp file with sentinel content for non-truncation tests.
fn sentinel_outfile(content: &str) -> tempfile::NamedTempFile {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(content.as_bytes()).unwrap();
    tmp
}

#[test]
fn command_clust_cc_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile("preserve me");
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["clust", "cc", "/nonexistent/path/to/pairs.tsv"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    let preserved = std::fs::read_to_string(out_path).unwrap();
    assert_eq!(preserved, "preserve me");
}

#[test]
fn command_clust_dbscan_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile("preserve me");
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) = run_with_outfile(
        &[
            "clust",
            "dbscan",
            "/nonexistent/path/to/pairs.tsv",
            "--eps",
            "0.05",
            "--min-points",
            "2",
        ],
        out_path,
    );
    assert!(!success, "expected failure for nonexistent input");
    let preserved = std::fs::read_to_string(out_path).unwrap();
    assert_eq!(preserved, "preserve me");
}

#[test]
fn command_clust_kmedoids_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile("preserve me");
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) = run_with_outfile(
        &[
            "clust",
            "k-medoids",
            "/nonexistent/path/to/pairs.tsv",
            "-k",
            "2",
            "--seed",
            "42",
        ],
        out_path,
    );
    assert!(!success, "expected failure for nonexistent input");
    let preserved = std::fs::read_to_string(out_path).unwrap();
    assert_eq!(preserved, "preserve me");
}

#[test]
fn command_clust_kmedoids_k_too_large_does_not_truncate_outfile() {
    // Input loads fine but k > n validation must bail before the writer is
    // opened, so the outfile is not truncated.
    let temp = tempfile::TempDir::new().unwrap();
    let input = temp.path().join("pairs.tsv");
    std::fs::write(&input, "A\tB\t0.1\n").unwrap();

    let existing = sentinel_outfile("preserve me");
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) = run_with_outfile(
        &["clust", "k-medoids", input.to_str().unwrap(), "-k", "10"],
        out_path,
    );
    assert!(!success, "expected failure for k > n");
    let preserved = std::fs::read_to_string(out_path).unwrap();
    assert_eq!(preserved, "preserve me");
}

#[test]
fn command_clust_mcl_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile("preserve me");
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) = run_with_outfile(
        &[
            "clust",
            "mcl",
            "/nonexistent/path/to/similarities.tsv",
            "--inflation",
            "2.0",
        ],
        out_path,
    );
    assert!(!success, "expected failure for nonexistent input");
    let preserved = std::fs::read_to_string(out_path).unwrap();
    assert_eq!(preserved, "preserve me");
}
