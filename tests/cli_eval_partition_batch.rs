use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_workflow_scan_eval_silhouette() -> anyhow::Result<()> {
    // 1. Prepare Tree
    // ((A:0.1,B:0.1):0.4,(C:0.1,D:0.1):0.4);
    // Heights: Leaves=0, (AB)=0.1, (CD)=0.1, Root=0.5
    let mut tree_file = NamedTempFile::new()?;
    write!(tree_file, "((A:0.1,B:0.1):0.4,(C:0.1,D:0.1):0.4);")?;

    // 2. Prepare Matrix
    // A-B: 0.2
    // C-D: 0.2
    // Others: 1.0
    let mut matrix_file = NamedTempFile::new()?;
    write!(
        matrix_file,
        "4
A 0.0 0.2 1.0 1.0
B 0.2 0.0 1.0 1.0
C 1.0 1.0 0.0 0.2
D 1.0 1.0 0.2 0.0
"
    )?;

    // 3. Run cut scan-simple
    // Scan range: 0.05 (below 0.1), 0.2 (between 0.1 and 0.5), 0.6 (above 0.5)
    // Actually scan logic: start, end, step.
    // 0.0, 0.6, 0.2 -> 0.0, 0.2, 0.4, 0.6
    // 0.0: Cut at 0.0. (AB) height 0.1 > 0. Cut. -> {A},{B},{C},{D}.
    // 0.2: Cut at 0.2. (AB) height 0.1 <= 0.2. Keep. Root height 0.5 > 0.2. Cut. -> {A,B}, {C,D}.
    // 0.4: Cut at 0.4. Same as 0.2.
    // 0.6: Cut at 0.6. Root height 0.5 <= 0.6. Keep. -> {A,B,C,D}.

    let mut cmd_cut = Command::cargo_bin("necom")?;
    let output_cut = cmd_cut
        .arg("cut")
        .arg("scan-simple")
        .arg(tree_file.path())
        .arg("--height")
        .arg("--range")
        .arg("0.0,0.6,0.2")
        .output()?;

    assert!(output_cut.status.success());
    let stdout_cut = String::from_utf8(output_cut.stdout)?;

    // Verify Long Format
    assert!(stdout_cut.starts_with("Group\tClusterID\tSampleID"));
    // Check some content
    // Threshold 0: 4 clusters.
    // Threshold 0.2: 2 clusters.

    // 4. Run clust eval (Batch)
    // Pass stdout_cut to stdin? Or write to file.
    // Let's write to file to be safe.
    let mut partitions_file = NamedTempFile::new()?;
    write!(partitions_file, "{}", stdout_cut)?;

    let mut cmd_eval = Command::cargo_bin("necom")?;
    let output_eval = cmd_eval
        .arg("eval")
        .arg("partition")
        .arg(partitions_file.path())
        .arg("--input-format")
        .arg("long")
        .arg("--matrix")
        .arg(matrix_file.path())
        .output()?;

    assert!(output_eval.status.success());
    let stdout_eval = String::from_utf8(output_eval.stdout)?;

    // Output:
    // Group\tsilhouette\tdunn\tc_index\tgamma\ttau
    // height=0\t0.000000\t...\t...
    // height=0.2\t0.800000\t...\t...
    // height=0.4\t0.800000\t...\t...
    // height=0.6\t0.000000\t...\t...

    let lines: Vec<&str> = stdout_eval.lines().collect();
    // Parse output
    // Header
    assert_eq!(lines[0], "Group\tsilhouette\tdunn\tc_index\tgamma\ttau");

    // Rows
    for line in &lines[1..] {
        let parts: Vec<&str> = line.split('\t').collect();
        let group = parts[0];
        let sil: f64 = parts[1].parse()?;

        // Parse group "height=val"
        let val_str = group.strip_prefix("height=").unwrap();
        let val: f64 = val_str.parse()?;

        if (val - 0.0).abs() < 1e-6 {
            assert!((sil - 0.0).abs() < 1e-6);
        } else if (val - 0.2).abs() < 1e-6 || (val - 0.4).abs() < 1e-6 {
            assert!((sil - 0.8).abs() < 1e-6);
        } else if (val - 0.6).abs() < 1e-6 {
            assert!((sil - 0.0).abs() < 1e-6);
        }
    }

    Ok(())
}

#[test]
fn test_eval_partition_batch_other_cluster_format() -> anyhow::Result<()> {
    // Regression test: previously the batch-mode `--other` was hardcoded to
    // `PartitionFormat::Pair`, so a cluster-format truth file was silently
    // mis-parsed. With the fix, the user-specified --input-format applies to
    // both p1 (Long) and --other (Cluster).
    //
    // Long partition file (4 groups, each is the same partition {A,B},{C,D}):
    let long_content = "Group\tClusterID\tSampleID\n\
g1\t1\tA\n\
g1\t1\tB\n\
g1\t2\tC\n\
g1\t2\tD\n\
g2\t1\tA\n\
g2\t1\tB\n\
g2\t2\tC\n\
g2\t2\tD\n";
    let mut long_file = NamedTempFile::new()?;
    write!(long_file, "{}", long_content)?;

    // Cluster-format truth file:
    let truth_content = "A B\nC D\n";
    let mut truth_file = NamedTempFile::new()?;
    write!(truth_file, "{}", truth_content)?;

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(long_file.path())
        .arg("--input-format")
        .arg("long")
        .arg("--other")
        .arg(truth_file.path())
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    // Header: Group + external metric names
    assert!(lines[0].starts_with("Group\t"));
    assert!(lines[0].contains("ari"));

    // Each group is identical to truth, so ARI should be 1.0.
    for line in &lines[1..] {
        let parts: Vec<&str> = line.split('\t').collect();
        let ari: f64 = parts[1].parse()?;
        assert!(
            (ari - 1.0).abs() < 1e-6,
            "Expected ARI=1.0 for perfect match, got {} in line: {}",
            ari,
            line
        );
    }

    Ok(())
}

#[test]
fn test_eval_partition_batch_conflict_other_matrix() -> anyhow::Result<()> {
    // Mutual exclusion in batch mode: --other and --matrix together must error.
    let long_content = "Group\tClusterID\tSampleID\n\
g1\t1\tA\n\
g1\t1\tB\n";
    let mut long_file = NamedTempFile::new()?;
    write!(long_file, "{}", long_content)?;

    let mut truth_file = NamedTempFile::new()?;
    writeln!(truth_file, "A B")?;

    let mut matrix_file = NamedTempFile::new()?;
    writeln!(matrix_file, "2\nA 0.0 1.0\nB 1.0 0.0")?;

    let mut cmd = Command::cargo_bin("necom")?;
    let output = cmd
        .arg("eval")
        .arg("partition")
        .arg(long_file.path())
        .arg("--input-format")
        .arg("long")
        .arg("--other")
        .arg(truth_file.path())
        .arg("--matrix")
        .arg(matrix_file.path())
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure when both --other and --matrix are provided in batch mode"
    );
    assert!(
        stderr.contains("only one of"),
        "Expected mutual exclusion error, got: {}",
        stderr
    );

    Ok(())
}
