#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_eval_replicate() {
    // 1. Create target tree
    let mut target_file = NamedTempFile::new().unwrap();
    writeln!(target_file, "((A,B),(C,D));").unwrap();

    // 2. Create replicate trees
    let mut replicates_file = NamedTempFile::new().unwrap();
    writeln!(replicates_file, "((A,B),(C,D));").unwrap();
    writeln!(replicates_file, "((A,B),(C,D));").unwrap();
    writeln!(replicates_file, "((A,C),(B,D));").unwrap(); // different topology

    // 3. Run command (absolute counts)
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "eval",
            "replicate",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
        ])
        .run();

    assert!(stdout.contains("((A,B)2,(C,D)2);"));

    // 4. Run command (percent)
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "eval",
            "replicate",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
            "--percent",
        ])
        .run();

    // 2/3 * 100 = 66
    assert!(stdout.contains("((A,B)66,(C,D)66);"));
}

#[test]
fn test_eval_replicate_overwrites_internal_labels() {
    // Target tree has an internal label "OldLabel" that should be overwritten.
    let mut target_file = NamedTempFile::new().unwrap();
    writeln!(target_file, "((A,B)OldLabel,(C,D));").unwrap();

    let mut replicates_file = NamedTempFile::new().unwrap();
    writeln!(replicates_file, "((A,B),(C,D));").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "eval",
            "replicate",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
        ])
        .run();

    assert!(!stdout.contains("OldLabel"));
    assert!(stdout.contains("((A,B)1,(C,D)1);"));
}

#[test]
fn test_eval_replicate_preserves_root_label_by_default() {
    let mut target_file = NamedTempFile::new().unwrap();
    writeln!(target_file, "((A,B),(C,D))Root;").unwrap();

    let mut replicates_file = NamedTempFile::new().unwrap();
    writeln!(replicates_file, "((A,B),(C,D));").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "eval",
            "replicate",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
        ])
        .run();

    assert!(
        stdout.contains("Root"),
        "root label should be preserved by default"
    );
}

#[test]
fn test_eval_replicate_override_root_label() {
    let mut target_file = NamedTempFile::new().unwrap();
    writeln!(target_file, "((A,B),(C,D))Root;").unwrap();

    let mut replicates_file = NamedTempFile::new().unwrap();
    writeln!(replicates_file, "((A,B),(C,D));").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "eval",
            "replicate",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
            "--override-root",
        ])
        .run();

    assert!(!stdout.contains("Root"), "root label should be overridden");
    assert!(
        stdout.contains("((A,B)1,(C,D)1)1;"),
        "root should get support value 1"
    );
}

#[test]
fn test_eval_replicate_outfile_not_truncated_on_empty_replicates() {
    // If the command fails before producing output, an existing outfile must
    // not be truncated. Regression test for opening the writer too early.
    let mut existing = NamedTempFile::new().unwrap();
    existing.write_all(b"preserve me").unwrap();
    let out_path = existing.path().to_str().unwrap();

    let mut target_file = NamedTempFile::new().unwrap();
    writeln!(target_file, "((A,B),(C,D));").unwrap();

    let replicates_file = NamedTempFile::new().unwrap();
    // Truly empty file should trigger "No replicate trees found".

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "eval",
            "replicate",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
            "--outfile",
            out_path,
        ])
        .run_fail();

    assert!(
        stderr.contains("No replicate trees found"),
        "expected empty replicates error, got stderr: {}",
        stderr
    );

    let preserved = std::fs::read_to_string(out_path).unwrap();
    assert_eq!(preserved, "preserve me");
}

#[test]
fn test_eval_replicate_rejects_empty_target_file() {
    let target_file = NamedTempFile::new().unwrap();

    let mut replicates_file = NamedTempFile::new().unwrap();
    writeln!(replicates_file, "((A,B),(C,D));").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "eval",
            "replicate",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
        ])
        .run_fail();

    assert!(
        stderr.contains("No target trees found"),
        "expected empty target error, got stderr: {}",
        stderr
    );
}

#[test]
fn test_eval_replicate_rejects_mismatched_target_leaf_sets() {
    let mut target_file = NamedTempFile::new().unwrap();
    writeln!(target_file, "((A,B),(C,D));").unwrap();

    let mut replicates_file = NamedTempFile::new().unwrap();
    writeln!(replicates_file, "((A,B),(C,E));").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "eval",
            "replicate",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
        ])
        .run_fail();

    assert!(
        stderr.contains("target tree 1 leaf set differs from replicate trees"),
        "expected target leaf-set mismatch error, got stderr: {}",
        stderr
    );
}

#[test]
fn test_eval_replicate_rejects_mismatched_replicate_leaf_sets() {
    let mut target_file = NamedTempFile::new().unwrap();
    writeln!(target_file, "(A,B);").unwrap();

    let mut replicates_file = NamedTempFile::new().unwrap();
    writeln!(replicates_file, "(A,B);").unwrap();
    writeln!(replicates_file, "(A,C);").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "eval",
            "replicate",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
        ])
        .run_fail();

    assert!(
        stderr.contains("replicate tree 2 leaf set differs from first replicate"),
        "expected clear error for mismatched replicate leaf sets, got stderr: {}",
        stderr
    );
}
