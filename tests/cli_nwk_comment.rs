#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;
use std::io::Write;
use tempfile::Builder;

#[test]
fn command_nwk_comment_dot_default_color() {
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file, "(A,B);").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "comment",
            file.path().to_str().unwrap(),
            "--node",
            "A",
            "--dot",
        ])
        .run();

    assert!(stdout.contains(":dot=black"));
}

#[test]
fn command_nwk_comment_dot_custom_color() {
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file, "(A,B);").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "comment",
            file.path().to_str().unwrap(),
            "--node",
            "A",
            "--dot",
            "red",
        ])
        .run();

    assert!(stdout.contains(":dot=red"));
    assert!(!stdout.contains(":dot=black"));
}

#[test]
fn command_nwk_comment_no_dot_leaves_node_unchanged() {
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file, "(A,B);").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "comment",
            file.path().to_str().unwrap(),
            "--node",
            "A",
        ])
        .run();

    assert!(!stdout.contains(":dot"));
}
