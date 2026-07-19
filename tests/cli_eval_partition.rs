use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_clust_eval_perfect_match() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/clust/perfect_1.tsv")
        .arg("--other")
        .arg("tests/clust/perfect_2.tsv")
        .arg("--input-format")
        .arg("cluster")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    assert!(output.status.success());

    // Check header
    assert!(stdout.contains("ari\tami\thomogeneity\tcompleteness\tv_measure\tfmi\tnmi\tmi\tri\tjaccard\tprecision\trecall"));

    // Check values (should be 1.0)
    // The second line should be numbers
    let lines: Vec<&str> = stdout.lines().collect();
    let values: Vec<&str> = lines[1].split_whitespace().collect();

    assert_eq!(values.len(), 12);
    // ARI
    assert!((values[0].parse::<f64>()? - 1.0).abs() < 1e-6);
    // AMI
    assert!((values[1].parse::<f64>()? - 1.0).abs() < 1e-6);
    // V-Measure
    assert!((values[4].parse::<f64>()? - 1.0).abs() < 1e-6);
    // FMI
    assert!((values[5].parse::<f64>()? - 1.0).abs() < 1e-6);
    // NMI
    assert!((values[6].parse::<f64>()? - 1.0).abs() < 1e-6);

    Ok(())
}

#[test]
fn test_clust_eval_no_singletons() -> anyhow::Result<()> {
    // Create temporary files
    // Truth: {A, B, C} (Cluster 1), {D} (Singleton), {E} (Singleton)
    // Pred: {A, B, C} (Cluster 1), {D, E} (Cluster 2)
    // Format: ClusterID <tab> Item

    let truth_content = "1\tA\n1\tB\n1\tC\n2\tD\n3\tE\n";
    let pred_content = "1\tA\n1\tB\n1\tC\n2\tD\n2\tE\n";

    let mut truth_file = NamedTempFile::new()?;
    let mut pred_file = NamedTempFile::new()?;
    truth_file.write_all(truth_content.as_bytes())?;
    pred_file.write_all(pred_content.as_bytes())?;
    let truth_path = truth_file.path().to_str().unwrap();
    let pred_path = pred_file.path().to_str().unwrap();

    // 1. Run WITHOUT --no-singletons
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(pred_path)
        .arg("--other")
        .arg(truth_path)
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    let values: Vec<&str> = lines[1].split_whitespace().collect();
    let ari = values[0].parse::<f64>()?;
    // ARI should be < 1.0
    assert!(ari < 0.99, "ARI was {} (expected < 0.99)", ari);

    // 2. Run WITH --no-singletons
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(pred_path)
        .arg("--other")
        .arg(truth_path)
        .arg("--no-singletons")
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    let values: Vec<&str> = lines[1].split_whitespace().collect();
    let ari = values[0].parse::<f64>()?;

    // ARI should be 1.0
    assert!((ari - 1.0).abs() < 1e-6, "ARI was {}", ari);

    Ok(())
}

#[test]
fn test_clust_eval_disjoint() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/clust/perfect_1.tsv")
        .arg("--other")
        .arg("tests/clust/disjoint_2.tsv")
        .arg("--input-format")
        .arg("cluster")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    assert!(output.status.success());

    let lines: Vec<&str> = stdout.lines().collect();
    let values: Vec<&str> = lines[1].split_whitespace().collect();

    // ARI should be -0.5
    assert!((values[0].parse::<f64>()? - (-0.5)).abs() < 1e-6);
    // AMI should be -0.5 (approx, due to log calculation precision?)
    // In my manual calculation, it was exactly -0.5.
    // Let's check roughly.
    let ami = values[1].parse::<f64>()?;
    assert!((ami - (-0.5)).abs() < 1e-4, "AMI was {}", ami);

    Ok(())
}

#[test]
fn test_clust_eval_single_vs_singletons() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/clust/single_1.tsv")
        .arg("--other")
        .arg("tests/clust/singletons.tsv")
        .arg("--input-format")
        .arg("cluster")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    assert!(output.status.success());

    let lines: Vec<&str> = stdout.lines().collect();
    let values: Vec<&str> = lines[1].split_whitespace().collect();

    // ARI = 0
    assert!((values[0].parse::<f64>()? - 0.0).abs() < 1e-6);
    // AMI = 0
    assert!((values[1].parse::<f64>()? - 0.0).abs() < 1e-6);

    Ok(())
}

#[test]
fn test_clust_eval_pair_format() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/clust/pair_1.tsv")
        .arg("--other")
        .arg("tests/clust/pair_2.tsv")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    assert!(output.status.success());

    // Check header
    assert!(stdout.contains("ari\tami\thomogeneity\tcompleteness\tv_measure\tfmi\tnmi\tmi\tri\tjaccard\tprecision\trecall"));

    // Check values (should be 1.0)
    let lines: Vec<&str> = stdout.lines().collect();
    let values: Vec<&str> = lines[1].split_whitespace().collect();

    assert_eq!(values.len(), 12);
    // ARI
    assert!((values[0].parse::<f64>()? - 1.0).abs() < 1e-6);
    // AMI
    assert!((values[1].parse::<f64>()? - 1.0).abs() < 1e-6);

    Ok(())
}

#[test]
fn test_clust_eval_internal_silhouette() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/clust/eval/simple.pair")
        .arg("--matrix")
        .arg("tests/clust/eval/simple.matrix.phy")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    assert!(output.status.success());

    // Output format:
    // silhouette   dunn    c_index   gamma   tau
    // 0.5167   4.0000  0.0  0.9707  0.8165

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "silhouette\tdunn\tc_index\tgamma\ttau");

    let values: Vec<&str> = lines[1].split_whitespace().collect();
    let score = values[0].parse::<f64>()?;
    let expected = 1.55 / 3.0;
    assert!(
        (score - expected).abs() < 1e-4,
        "Score was {}, expected {}",
        score,
        expected
    );

    let dunn = values[1].parse::<f64>()?;
    assert!((dunn - 4.0).abs() < 1e-4, "Dunn was {}", dunn);

    let c_index = values[2].parse::<f64>()?;
    // Simple dataset: 3 points. A-B in C1, C in C2.
    // Dists: A-B=0.5, A-C=2.0, B-C=2.5.
    // N_W = 1 (A-B). S_W = 0.5.
    // S_min (sum of smallest 1) = 0.5.
    // S_max (sum of largest 1) = 2.5.
    // C = (0.5-0.5)/(2.5-0.5) = 0.
    assert!((c_index - 0.0).abs() < 1e-4, "C-index was {}", c_index);

    let gamma = values[3].parse::<f64>()?;
    assert!((gamma - 0.970725).abs() < 1e-4, "Gamma was {}", gamma);

    let tau = values[4].parse::<f64>()?;
    assert!((tau - 0.816497).abs() < 1e-4, "Tau was {}", tau);

    Ok(())
}

#[test]
fn test_clust_eval_internal_db() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/clust/eval/db.pair")
        .arg("--coords")
        .arg("tests/clust/eval/db.coords.tsv")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    assert!(output.status.success());

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines[0],
        "davies_bouldin\tcalinski_harabasz\tpbm\tball_hall\txie_beni\twemmert_gancarski"
    );

    let values: Vec<&str> = lines[1].split_whitespace().collect();
    let score = values[0].parse::<f64>()?;
    let expected = 0.2;
    assert!(
        (score - expected).abs() < 1e-4,
        "Score was {}, expected {}",
        score,
        expected
    );

    let ch = values[1].parse::<f64>()?;
    assert!((ch - 50.0).abs() < 1e-4, "CH was {}", ch);

    assert_eq!(values.len(), 6);

    Ok(())
}

#[test]
fn test_clust_eval_invariance_sample_order() -> anyhow::Result<()> {
    let content_orig = "1\tA\n1\tB\n2\tC\n2\tD\n";
    let content_shuffled = "1\tB\n2\tD\n1\tA\n2\tC\n";

    let mut orig_file = NamedTempFile::new()?;
    let mut shuffled_file = NamedTempFile::new()?;
    orig_file.write_all(content_orig.as_bytes())?;
    shuffled_file.write_all(content_shuffled.as_bytes())?;
    let orig_path = orig_file.path().to_str().unwrap();
    let shuffled_path = shuffled_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(orig_path)
        .arg("--other")
        .arg(shuffled_path)
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    let values: Vec<&str> = lines[1].split_whitespace().collect();

    assert!((values[0].parse::<f64>()? - 1.0).abs() < 1e-6);

    Ok(())
}

#[test]
fn test_clust_eval_invariance_label_scaling() -> anyhow::Result<()> {
    let content_small = "1\tA\n1\tB\n2\tC\n2\tD\n";
    let content_large = "100\tA\n100\tB\n200\tC\n200\tD\n";

    let mut small_file = NamedTempFile::new()?;
    let mut large_file = NamedTempFile::new()?;
    small_file.write_all(content_small.as_bytes())?;
    large_file.write_all(content_large.as_bytes())?;
    let small_path = small_file.path().to_str().unwrap();
    let large_path = large_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(small_path)
        .arg("--other")
        .arg(large_path)
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    let values: Vec<&str> = lines[1].split_whitespace().collect();

    assert!((values[0].parse::<f64>()? - 1.0).abs() < 1e-6);

    Ok(())
}

#[test]
fn test_clust_eval_empty_input() -> anyhow::Result<()> {
    let empty_file = NamedTempFile::new()?;
    let empty_path = empty_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(empty_path)
        .arg("--other")
        .arg(empty_path)
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() > 1 {
        let values: Vec<&str> = lines[1].split_whitespace().collect();
        assert_eq!(values[0], "0.000000");
    }

    Ok(())
}

#[test]
fn test_eval_malformed_pair_format() -> anyhow::Result<()> {
    let mut malformed_file = NamedTempFile::new()?;
    malformed_file.write_all("A\nB\n".as_bytes())?;
    let malformed_path = malformed_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(malformed_path)
        .arg("--other")
        .arg("tests/clust/eval/simple.pair")
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure for malformed pair format, but command succeeded"
    );
    assert!(
        stderr.to_lowercase().contains("pair")
            || stderr.to_lowercase().contains("invalid"),
        "Expected pair format error, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_eval_missing_other_for_external() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/clust/perfect_1.tsv")
        .arg("--input-format")
        .arg("cluster")
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure when no evaluation target is provided"
    );
    assert!(
        stderr.to_lowercase().contains("other")
            || stderr.to_lowercase().contains("must be provided"),
        "Expected missing evaluation target error, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_eval_partition_conflict_matrix_tree() -> anyhow::Result<()> {
    // Mutual exclusion: providing both --matrix and --tree must error out
    // rather than silently dropping one target.
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/clust/eval/simple.pair")
        .arg("--matrix")
        .arg("tests/clust/eval/simple.matrix.phy")
        .arg("--tree")
        .arg("tests/clust/eval/simple.matrix.phy") // any file, just to satisfy clap
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure when both --matrix and --tree are provided"
    );
    assert!(
        stderr.contains("only one of"),
        "Expected mutual exclusion error, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_eval_partition_conflict_other_coords() -> anyhow::Result<()> {
    // Mutual exclusion: providing both --other and --coords must error out.
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/clust/eval/simple.pair")
        .arg("--other")
        .arg("tests/clust/eval/simple.pair")
        .arg("--coords")
        .arg("tests/clust/eval/db.coords.tsv")
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure when both --other and --coords are provided"
    );
    assert!(
        stderr.contains("only one of"),
        "Expected mutual exclusion error, got: {}",
        stderr
    );

    Ok(())
}
