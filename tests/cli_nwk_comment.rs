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
fn command_nwk_comment_string_with_reserved_chars_round_trip() {
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file, "(A,B);").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "comment",
            file.path().to_str().unwrap(),
            "--node",
            "A",
            "--string",
            "x]y\\z",
        ])
        .run();

    // The reserved characters must be escaped in the Newick output.
    assert!(stdout.contains(":string=x\\]y\\\\z"));

    // The generated Newick must remain parseable.
    let mut out_file = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(out_file, "{}", stdout.trim()).unwrap();
    NecomCmd::new()
        .args(&["nwk", "info", out_file.path().to_str().unwrap()])
        .run();
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
