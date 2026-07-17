#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_nwk_support() {
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
            "nwk",
            "support",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
        ])
        .run();

    assert!(stdout.contains("((A,B)2,(C,D)2);"));

    // 4. Run command (percent)
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "support",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
            "--percent",
        ])
        .run();

    // 2/3 * 100 = 66
    assert!(stdout.contains("((A,B)66,(C,D)66);"));
}

#[test]
fn test_nwk_support_overwrites_internal_labels() {
    // Target tree has an internal label "OldLabel" that should be overwritten.
    let mut target_file = NamedTempFile::new().unwrap();
    writeln!(target_file, "((A,B)OldLabel,(C,D));").unwrap();

    let mut replicates_file = NamedTempFile::new().unwrap();
    writeln!(replicates_file, "((A,B),(C,D));").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "support",
            target_file.path().to_str().unwrap(),
            replicates_file.path().to_str().unwrap(),
        ])
        .run();

    assert!(!stdout.contains("OldLabel"));
    assert!(stdout.contains("((A,B)1,(C,D)1);"));
}

#[test]
fn test_nwk_support_preserves_root_label_by_default() {
    let mut target_file = NamedTempFile::new().unwrap();
    writeln!(target_file, "((A,B),(C,D))Root;").unwrap();

    let mut replicates_file = NamedTempFile::new().unwrap();
    writeln!(replicates_file, "((A,B),(C,D));").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "support",
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
fn test_nwk_support_override_root_label() {
    let mut target_file = NamedTempFile::new().unwrap();
    writeln!(target_file, "((A,B),(C,D))Root;").unwrap();

    let mut replicates_file = NamedTempFile::new().unwrap();
    writeln!(replicates_file, "((A,B),(C,D));").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "support",
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
