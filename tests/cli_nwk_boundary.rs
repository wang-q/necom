#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;

// ================================================================================================
// Boundary and regression tests for necom nwk
// ================================================================================================

#[test]
fn command_empty_stdin() {
    // Empty stdin should produce a friendly parse error, not a panic.
    let (_, stderr) = NecomCmd::new()
        .args(&["nwk", "stat", "stdin"])
        .stdin("")
        .run_fail();

    assert!(stderr.contains("error") || stderr.contains("Error") || stderr.contains("parse"));
}

#[test]
fn command_missing_semicolon() {
    // Missing semicolon should produce a parse error, not a panic.
    let (_, stderr) = NecomCmd::new()
        .args(&["nwk", "stat", "stdin"])
        .stdin("(A,B)C")
        .run_fail();

    assert!(
        stderr.contains("error")
            || stderr.contains("Error")
            || stderr.contains("parse")
            || stderr.contains(";")
    );
}

#[test]
fn command_single_node_round_trip() {
    // A single-node tree should round-trip cleanly.
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "stdin", "--compact"])
        .stdin("A;")
        .run();

    assert_eq!(stdout.trim(), "A;");
}

#[test]
fn command_non_finite_branch_lengths_omitted() {
    // NaN, infinity, negative, and zero branch lengths should all be omitted on output.
    let input = "(A:NaN,B:inf,C:-inf,D:-1.0,E:0.0)Root;";
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "stdin", "--compact"])
        .stdin(input)
        .run();

    assert_eq!(stdout.trim(), "(A,B,C,D,E)Root;");
}

#[test]
fn command_stat_empty_tree_file() {
    // A file with only whitespace should report an error, not panic.
    let (_, stderr) = NecomCmd::new()
        .args(&["nwk", "stat", "stdin"])
        .stdin("   \n\n   ")
        .run_fail();

    assert!(stderr.contains("error") || stderr.contains("Error") || stderr.contains("parse"));
}

#[test]
fn command_unbalanced_parens() {
    // Unbalanced parentheses should produce a parse error, not panic.
    let (_, stderr) = NecomCmd::new()
        .args(&["nwk", "stat", "stdin"])
        .stdin("(A,B;")
        .run_fail();

    assert!(
        stderr.contains("error")
            || stderr.contains("Error")
            || stderr.contains("parse")
            || stderr.contains("(")
    );
}

#[test]
fn command_distance_uses_first_tree_only() {
    // distance processes the first tree and ignores the rest without error.
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "distance", "stdin"])
        .stdin("(A:1,B:2)Root;(C,D)Other;")
        .run();

    assert!(stdout.contains("A"));
    assert!(stdout.contains("B"));
    assert!(!stdout.contains("Other"));
}

#[test]
fn command_nested_top_level_comments() {
    // Nested or escaped top-level comments should be skipped by the parser.
    let input = "[header [nested] end] (A,B)R; [tail \\] ok] (C,D)S;";
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "stdin", "--compact"])
        .stdin(input)
        .run();

    assert!(stdout.contains("R"));
    assert!(stdout.contains("S"));
    assert!(!stdout.contains("header"));
    assert!(!stdout.contains("tail"));
}
