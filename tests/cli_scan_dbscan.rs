#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;

#[test]
fn command_scan_dbscan_summary() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "scan-dbscan",
            "tests/clust/IBPA.fa.tsv",
            "--scan",
            "0.01,0.05,0.01",
            "--min-points",
            "2",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "Epsilon\tClusters\tNoise\tSilhouette\tDBIndex");
    assert!(lines.len() > 1, "expected at least one scan row");

    // Verify a data row has the expected number of tab-separated fields.
    let fields: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(fields.len(), 5);
    assert!(fields[0].parse::<f64>().is_ok());
    assert!(fields[1].parse::<usize>().is_ok());
    assert!(fields[2].parse::<usize>().is_ok());
    // Silhouette and DBIndex may be NA or finite.
    assert!(fields[3] == "NA" || fields[3].parse::<f64>().is_ok());
    assert!(fields[4] == "NA" || fields[4].parse::<f64>().is_ok());
}

#[test]
fn command_scan_dbscan_opt_eps_silhouette() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "scan-dbscan",
            "tests/clust/IBPA.fa.tsv",
            "--scan",
            "0.01,0.05,0.01",
            "--min-points",
            "2",
            "--opt-eps",
            "silhouette",
        ])
        .run();

    // Output should be a clustering partition, not the scan summary header.
    assert!(!stdout.starts_with("Epsilon\tClusters"));
    assert!(!stdout.is_empty());
    // At least one cluster line should be present.
    assert!(stdout.lines().count() > 0);
}

#[test]
fn command_scan_dbscan_opt_eps_pair() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "scan-dbscan",
            "tests/clust/IBPA.fa.tsv",
            "--scan",
            "0.01,0.05,0.01",
            "--min-points",
            "2",
            "--opt-eps",
            "max-clusters",
            "--format",
            "pair",
        ])
        .run();

    // Pair format lines: representative\tmember.
    for line in stdout.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            fields.len(),
            2,
            "pair line should have two columns: {}",
            line
        );
    }
}

#[test]
fn command_scan_dbscan_min_pct() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "clust",
            "scan-dbscan",
            "tests/clust/IBPA.fa.tsv",
            "--scan",
            "0.01,0.05,0.01",
            "--min-pct",
            "0.1",
        ])
        .run();

    assert_eq!(
        stdout.lines().next().unwrap(),
        "Epsilon\tClusters\tNoise\tSilhouette\tDBIndex"
    );
    assert!(stdout.lines().count() > 1);
}

#[test]
fn command_scan_dbscan_min_pct_conflicts_with_min_points() {
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "clust",
            "scan-dbscan",
            "tests/clust/IBPA.fa.tsv",
            "--scan",
            "0.01,0.05,0.01",
            "--min-points",
            "2",
            "--min-pct",
            "0.1",
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
fn command_scan_dbscan_fuzz_random_matrices() {
    use rand::{Rng, SeedableRng};
    use std::collections::HashSet;

    let mut rng = rand::rngs::StdRng::seed_from_u64(20260721);

    for _ in 0..20 {
        let n = rng.random_range(5..=15);
        let min_points = rng.random_range(2..=n.max(2));

        let names: Vec<String> = (0..n).map(|i| format!("s{}", i)).collect();
        let mut lines = Vec::new();
        for i in 0..n {
            for j in i + 1..n {
                let d = rng.random_range(0.0..1.0);
                lines.push(format!("{}\t{}\t{:.6}", names[i], names[j], d));
            }
        }

        let temp = tempfile::TempDir::new().unwrap();
        let input = temp.path().join("pairs.tsv");
        std::fs::write(&input, lines.join("\n") + "\n").unwrap();

        // Summary scan must not panic and must produce valid TSV rows.
        let (stdout, stderr) = NecomCmd::new()
            .args(&[
                "clust",
                "scan-dbscan",
                input.to_str().unwrap(),
                "--scan",
                "0.05,0.95,0.05",
                "--min-points",
                &min_points.to_string(),
            ])
            .run();

        assert!(
            stderr.is_empty() || !stderr.to_lowercase().contains("error"),
            "scan-dbscan produced error: {}",
            stderr
        );

        let mut rows = stdout.lines();
        assert_eq!(
            rows.next().unwrap(),
            "Epsilon\tClusters\tNoise\tSilhouette\tDBIndex"
        );

        let mut prev_eps: Option<f64> = None;
        let mut prev_noise: Option<usize> = None;
        for line in rows {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 5, "unexpected row: {}", line);

            let eps: f64 = fields[0].parse().expect("eps should be numeric");
            let clusters: usize = fields[1].parse().expect("clusters should be integer");
            let noise: usize = fields[2].parse().expect("noise should be integer");

            assert!(eps > 0.0, "eps must be positive: {}", eps);
            if let Some(prev) = prev_eps {
                assert!(eps >= prev, "eps values should be non-decreasing");
            }
            prev_eps = Some(eps);

            assert!(clusters <= n, "clusters should not exceed sample count");
            assert!(noise <= n, "noise should not exceed sample count");
            if let Some(prev) = prev_noise {
                assert!(
                    noise <= prev,
                    "noise count should be non-increasing as eps grows"
                );
            }
            prev_noise = Some(noise);

            for value in [&fields[3], &fields[4]] {
                if *value != "NA" {
                    let v: f64 = value.parse().expect("metric should be numeric or NA");
                    assert!(v.is_finite(), "metric should be finite or NA: {}", v);
                }
            }
        }

        // Opt-eps partition output must contain every sample exactly once.
        let (stdout_opt, stderr_opt) = NecomCmd::new()
            .args(&[
                "clust",
                "scan-dbscan",
                input.to_str().unwrap(),
                "--scan",
                "0.05,0.95,0.05",
                "--min-points",
                &min_points.to_string(),
                "--opt-eps",
                "silhouette",
                "--format",
                "pair",
            ])
            .run();

        assert!(
            stderr_opt.is_empty() || !stderr_opt.to_lowercase().contains("error"),
            "scan-dbscan --opt-eps produced error: {}",
            stderr_opt
        );

        let mut seen = HashSet::new();
        for line in stdout_opt.lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(
                fields.len(),
                2,
                "pair line should have two columns: {}",
                line
            );
            assert!(
                seen.insert(fields[1].to_string()),
                "sample appeared more than once: {}",
                line
            );
        }
        assert_eq!(
            seen.len(),
            n,
            "opt-eps partition should contain all {} samples",
            n
        );
    }
}
