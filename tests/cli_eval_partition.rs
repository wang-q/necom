use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_eval_partition_perfect_match() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/perfect_1.tsv")
        .arg("--other")
        .arg("tests/eval/perfect_2.tsv")
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
fn test_eval_partition_no_singletons() -> anyhow::Result<()> {
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
fn test_eval_partition_disjoint() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/perfect_1.tsv")
        .arg("--other")
        .arg("tests/eval/disjoint_2.tsv")
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
fn test_eval_partition_single_vs_singletons() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/single_1.tsv")
        .arg("--other")
        .arg("tests/eval/singletons.tsv")
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
fn test_eval_partition_truth_alias() -> anyhow::Result<()> {
    // `--truth` is a visible alias for `--other` and should behave identically.
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/perfect_1.tsv")
        .arg("--truth")
        .arg("tests/eval/perfect_2.tsv")
        .arg("--input-format")
        .arg("cluster")
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    let values: Vec<&str> = lines[1].split_whitespace().collect();
    assert!((values[0].parse::<f64>()? - 1.0).abs() < 1e-6);
    Ok(())
}

#[test]
fn test_eval_partition_other_format_single_mode() -> anyhow::Result<()> {
    // p1 is pair format (default), but --other is cluster format.
    // --other-format must override the default for --other.
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/pair_1.tsv")
        .arg("--other")
        .arg("tests/eval/perfect_2.tsv")
        .arg("--other-format")
        .arg("cluster")
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    let values: Vec<&str> = lines[1].split_whitespace().collect();
    assert!((values[0].parse::<f64>()? - 1.0).abs() < 1e-6);
    Ok(())
}

#[test]
fn test_eval_partition_pair_format() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/pair_1.tsv")
        .arg("--other")
        .arg("tests/eval/pair_2.tsv")
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
fn test_eval_partition_internal_silhouette() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/simple.pair")
        .arg("--matrix")
        .arg("tests/eval/simple.matrix.phy")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    assert!(output.status.success());

    // Output format:
    // silhouette   dunn    c_index   gamma   tau   davies_bouldin
    // 0.5167   4.0000  0.0  0.9707  0.8165  0.1250

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines[0],
        "silhouette\tdunn\tc_index\tgamma\ttau\tdavies_bouldin"
    );

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

    let db = values[5].parse::<f64>()?;
    assert!((db - 0.125).abs() < 1e-4, "Davies-Bouldin was {}", db);

    Ok(())
}

#[test]
fn test_eval_partition_internal_db() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/db.pair")
        .arg("--coords")
        .arg("tests/eval/db.coords.tsv")
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
fn test_eval_partition_invariance_sample_order() -> anyhow::Result<()> {
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
fn test_eval_partition_invariance_label_scaling() -> anyhow::Result<()> {
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
fn test_eval_partition_empty_input() -> anyhow::Result<()> {
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

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure for empty partition, but command succeeded"
    );
    assert!(
        stderr.contains("empty"),
        "Expected empty partition error, got: {}",
        stderr
    );

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
        .arg("tests/eval/simple.pair")
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
        .arg("tests/eval/perfect_1.tsv")
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
        .arg("tests/eval/simple.pair")
        .arg("--matrix")
        .arg("tests/eval/simple.matrix.phy")
        .arg("--tree")
        .arg("tests/eval/simple.matrix.phy") // any file, just to satisfy clap
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
fn test_eval_partition_disjoint_sample_sets() -> anyhow::Result<()> {
    // External evaluation requires the two partitions to cover the same sample
    // set. If p1 contains samples not in --other, the command must bail rather
    // than silently dropping them.
    let mut p1_file = NamedTempFile::new()?;
    p1_file.write_all("1\tA\n1\tB\n2\tC\n2\tD\n".as_bytes())?;
    let p1_path = p1_file.path().to_str().unwrap();

    let mut p2_file = NamedTempFile::new()?;
    p2_file.write_all("1\tA\n1\tB\n2\tC\n".as_bytes())?;
    let p2_path = p2_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(p1_path)
        .arg("--other")
        .arg(p2_path)
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure for disjoint sample sets, but command succeeded"
    );
    assert!(
        stderr.contains("sample sets do not match"),
        "Expected sample-set mismatch error, got: {}",
        stderr
    );
    assert!(
        stderr.contains("D"),
        "Error should mention the missing sample D, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_eval_partition_outfile_not_truncated_on_error() -> anyhow::Result<()> {
    // If the command fails before producing output, an existing outfile must
    // not be truncated. Regression test for opening the writer too early.
    let mut existing = NamedTempFile::new()?;
    existing.write_all(b"preserve me")?;
    let out_path = existing.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/perfect_1.tsv")
        .arg("--other")
        .arg("tests/eval/perfect_2.tsv")
        .arg("--matrix")
        .arg("tests/eval/simple.matrix.phy")
        .arg("--outfile")
        .arg(out_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("only one of"), "got stderr: {}", stderr);

    let preserved = std::fs::read_to_string(out_path)?;
    assert_eq!(preserved, "preserve me");

    Ok(())
}

#[test]
fn test_eval_partition_outfile_preserved_on_p1_load_failure() -> anyhow::Result<()> {
    // Regression test: when p1 fails to load (e.g., file does not exist), an
    // existing outfile must not be truncated. Before the fix, the writer was
    // opened before input loading, truncating the outfile on failure.
    let mut existing = NamedTempFile::new()?;
    existing.write_all(b"preserve me")?;
    let out_path = existing.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("/nonexistent/path/to/p1.tsv")
        .arg("--other")
        .arg("tests/eval/perfect_2.tsv")
        .arg("--input-format")
        .arg("cluster")
        .arg("--outfile")
        .arg(out_path)
        .output()?;

    assert!(
        !output.status.success(),
        "Expected failure for nonexistent p1, but command succeeded"
    );
    let preserved = std::fs::read_to_string(out_path)?;
    assert_eq!(preserved, "preserve me");

    Ok(())
}

#[test]
fn test_eval_partition_outfile_preserved_on_matrix_load_failure() -> anyhow::Result<()> {
    // Regression test: when --matrix fails to load (e.g., declared size
    // doesn't match the number of data rows), an existing outfile must not be
    // truncated. This exercises the single-mode code path where the matrix is
    // loaded after p1 succeeds.
    let mut existing = NamedTempFile::new()?;
    existing.write_all(b"preserve me")?;
    let out_path = existing.path().to_str().unwrap();

    let mut malformed_matrix = NamedTempFile::new()?;
    // Declares 4 sequences but only provides 2 data rows.
    malformed_matrix.write_all(b"4\nA\t0.0\t1.0\t2.0\t3.0\nB\t1.0\t0.0\t4.0\t5.0\n")?;
    let matrix_path = malformed_matrix.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/perfect_1.tsv")
        .arg("--input-format")
        .arg("cluster")
        .arg("--matrix")
        .arg(matrix_path)
        .arg("--outfile")
        .arg(out_path)
        .output()?;

    assert!(
        !output.status.success(),
        "Expected failure for malformed --matrix, but command succeeded"
    );
    let preserved = std::fs::read_to_string(out_path)?;
    assert_eq!(preserved, "preserve me");

    Ok(())
}

#[test]
fn test_eval_partition_outfile_preserved_on_batch_p1_load_failure() -> anyhow::Result<()>
{
    // Regression test for batch mode: when the Long-format p1 fails to load,
    // an existing outfile must not be truncated.
    let mut existing = NamedTempFile::new()?;
    existing.write_all(b"preserve me")?;
    let out_path = existing.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("/nonexistent/path/to/long.tsv")
        .arg("--input-format")
        .arg("long")
        .arg("--other")
        .arg("tests/eval/perfect_2.tsv")
        .arg("--outfile")
        .arg(out_path)
        .output()?;

    assert!(
        !output.status.success(),
        "Expected failure for nonexistent batch p1, but command succeeded"
    );
    let preserved = std::fs::read_to_string(out_path)?;
    assert_eq!(preserved, "preserve me");

    Ok(())
}

#[test]
fn test_eval_partition_conflict_other_coords() -> anyhow::Result<()> {
    // Mutual exclusion: providing both --other and --coords must error out.
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/simple.pair")
        .arg("--other")
        .arg("tests/eval/simple.pair")
        .arg("--coords")
        .arg("tests/eval/db.coords.tsv")
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

#[test]
fn test_eval_partition_matrix_missing_sample() -> anyhow::Result<()> {
    // Partition contains D, but the matrix only has A/B/C. This must bail
    // with a clear message instead of silently returning NaN metrics.
    let mut partition_file = NamedTempFile::new()?;
    partition_file.write_all("1\tA\n1\tB\n2\tC\n2\tD\n".as_bytes())?;
    let partition_path = partition_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(partition_path)
        .arg("--matrix")
        .arg("tests/eval/simple.matrix.phy")
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure for missing matrix sample, but command succeeded"
    );
    assert!(
        stderr.contains("missing in --matrix"),
        "Expected missing --matrix sample error, got: {}",
        stderr
    );
    assert!(
        stderr.contains("D"),
        "Error should mention the missing sample D, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_eval_partition_tree_missing_sample() -> anyhow::Result<()> {
    // Partition contains D, but the tree only has A/B/C. This must bail
    // with a clear message instead of silently returning NaN metrics.
    let mut partition_file = NamedTempFile::new()?;
    partition_file.write_all("1\tA\n1\tB\n2\tC\n2\tD\n".as_bytes())?;
    let partition_path = partition_file.path().to_str().unwrap();

    let mut tree_file = NamedTempFile::new()?;
    tree_file.write_all("((A:0.1,B:0.1):0.2,C:0.3);".as_bytes())?;
    let tree_path = tree_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(partition_path)
        .arg("--tree")
        .arg(tree_path)
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure for missing tree sample, but command succeeded"
    );
    assert!(
        stderr.contains("missing in --tree"),
        "Expected missing --tree sample error, got: {}",
        stderr
    );
    assert!(
        stderr.contains("D"),
        "Error should mention the missing sample D, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_eval_partition_coords_missing_sample() -> anyhow::Result<()> {
    // Partition contains X, but the coords file only has A/B/C/D. This must
    // bail with a clear message instead of silently dropping the sample.
    let mut partition_file = NamedTempFile::new()?;
    partition_file.write_all("1\tA\n1\tB\n2\tC\n2\tX\n".as_bytes())?;
    let partition_path = partition_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(partition_path)
        .arg("--coords")
        .arg("tests/eval/db.coords.tsv")
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure for missing coords sample, but command succeeded"
    );
    assert!(
        stderr.contains("missing in --coords"),
        "Expected missing --coords sample error, got: {}",
        stderr
    );
    assert!(
        stderr.contains("X"),
        "Error should mention the missing sample X, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_eval_partition_batch_matrix_missing_sample() -> anyhow::Result<()> {
    // Batch long format with a group containing sample E, but the matrix only
    // has A/B/C/D. This must bail with a clear message.
    let mut long_file = NamedTempFile::new()?;
    long_file.write_all("g1\t1\tA\ng1\t1\tB\ng1\t2\tC\ng1\t2\tE\n".as_bytes())?;
    let long_path = long_file.path().to_str().unwrap();

    let mut matrix_file = NamedTempFile::new()?;
    matrix_file.write_all(
        "4\nA\t0.0\t1.0\t2.0\t3.0\nB\t1.0\t0.0\t4.0\t5.0\nC\t2.0\t4.0\t0.0\t6.0\nD\t3.0\t5.0\t6.0\t0.0\n".as_bytes(),
    )?;
    let matrix_path = matrix_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(long_path)
        .arg("--input-format")
        .arg("long")
        .arg("--matrix")
        .arg(matrix_path)
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure for missing batch matrix sample, but command succeeded"
    );
    assert!(
        stderr.contains("missing in --matrix"),
        "Expected missing --matrix sample error, got: {}",
        stderr
    );
    assert!(
        stderr.contains("E"),
        "Error should mention the missing sample E, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_eval_partition_p1_stdin() -> anyhow::Result<()> {
    // p1 should accept "stdin" as the input path, consistent with
    // --matrix / --tree / --coords. Use the same partition as perfect_2.tsv
    // (Cluster 1={A,B}, Cluster 2={C,D}) so ARI must be 1.0.
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("stdin")
        .arg("--other")
        .arg("tests/eval/perfect_2.tsv")
        .arg("--input-format")
        .arg("cluster")
        .write_stdin("A\tB\nC\tD\n")
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines[0].starts_with("ari\tami"));
    let values: Vec<&str> = lines[1].split_whitespace().collect();
    assert!((values[0].parse::<f64>()? - 1.0).abs() < 1e-6);

    Ok(())
}

#[test]
fn test_eval_partition_other_stdin() -> anyhow::Result<()> {
    // --other should accept "stdin" as the input path, consistent with
    // --matrix / --tree / --coords. Use the same partition as perfect_1.tsv
    // (Cluster 1={A,B}, Cluster 2={C,D}) so ARI must be 1.0.
    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg("tests/eval/perfect_1.tsv")
        .arg("--other")
        .arg("stdin")
        .arg("--input-format")
        .arg("cluster")
        .write_stdin("A\tB\nC\tD\n")
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines[0].starts_with("ari\tami"));
    let values: Vec<&str> = lines[1].split_whitespace().collect();
    assert!((values[0].parse::<f64>()? - 1.0).abs() < 1e-6);

    Ok(())
}

#[test]
fn test_eval_partition_coords_stdin() -> anyhow::Result<()> {
    // --coords should accept "stdin" as the input path, consistent with
    // --matrix / --tree.
    let partition_content = "1\tA\n1\tB\n2\tC\n2\tD\n";
    let coords_content = "A\t0.0\t0.0\nB\t1.0\t0.0\nC\t5.0\t0.0\nD\t6.0\t0.0\n";

    let mut partition_file = NamedTempFile::new()?;
    partition_file.write_all(partition_content.as_bytes())?;
    let partition_path = partition_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(partition_path)
        .arg("--coords")
        .arg("stdin")
        .write_stdin(coords_content)
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines[0],
        "davies_bouldin\tcalinski_harabasz\tpbm\tball_hall\txie_beni\twemmert_gancarski"
    );
    // Sanity check: CH should be positive (well-separated compact clusters).
    let values: Vec<&str> = lines[1].split_whitespace().collect();
    assert!(values[1].parse::<f64>()? > 0.0);

    Ok(())
}

#[test]
fn test_eval_partition_other_empty_after_no_singletons() -> anyhow::Result<()> {
    // When --no-singletons removes every cluster from --other (all were
    // singletons), the command must bail with a clear error rather than
    // producing misleading all-zero metrics.
    let mut p1_file = NamedTempFile::new()?;
    p1_file.write_all("1\tA\n1\tB\n2\tC\n2\tD\n".as_bytes())?;
    let p1_path = p1_file.path().to_str().unwrap();

    // Every cluster in p2 is a singleton.
    let mut p2_file = NamedTempFile::new()?;
    p2_file.write_all("1\tA\n2\tB\n3\tC\n4\tD\n".as_bytes())?;
    let p2_path = p2_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(p1_path)
        .arg("--other")
        .arg(p2_path)
        .arg("--no-singletons")
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure when --other becomes empty after --no-singletons, but command succeeded"
    );
    assert!(
        stderr.contains("empty"),
        "Expected empty partition error, got: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_eval_partition_batch_other_empty_after_no_singletons() -> anyhow::Result<()> {
    // Batch mode: when --no-singletons removes every cluster from --other,
    // the command must bail with a clear error rather than emitting all-zero
    // rows for every group.
    //
    // In batch mode `--other` defaults to Cluster format (each line is a
    // cluster whose whitespace-separated items are members). To get true
    // singleton clusters we therefore use `--other-format pair`, where each
    // line is `<cluster_id>\t<sample>` and a cluster with one sample is a
    // singleton that `--no-singletons` will remove.
    let long_content = "Group\tClusterID\tSampleID\n\
g1\t1\tA\n\
g1\t1\tB\n";
    let mut long_file = NamedTempFile::new()?;
    long_file.write_all(long_content.as_bytes())?;

    // Every cluster in truth is a singleton (one sample per cluster ID).
    let mut truth_file = NamedTempFile::new()?;
    truth_file.write_all("1\tA\n2\tB\n".as_bytes())?;

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(long_file.path())
        .arg("--input-format")
        .arg("long")
        .arg("--other-format")
        .arg("pair")
        .arg("--other")
        .arg(truth_file.path())
        .arg("--no-singletons")
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure when --other becomes empty after --no-singletons in batch mode"
    );
    assert!(
        stderr.contains("empty"),
        "Expected empty partition error, got: {}",
        stderr
    );

    Ok(())
}
