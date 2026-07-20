// Regression tests for the "open writer before input validation" anti-pattern.
// Before the fix, all nwk subcommands opened the output writer before loading
// or validating input, which truncated the output file on failure. These tests
// verify that an existing outfile is preserved when the command fails.

use std::io::Write;

/// Pre-create a named temp file with sentinel content for non-truncation tests.
fn sentinel_outfile(content: &str) -> tempfile::NamedTempFile {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(content.as_bytes()).unwrap();
    tmp
}

/// Run `necom` with the given args, appending `--outfile <path>`.
/// Returns `(success, stdout, stderr)`.
fn run_with_outfile(args: &[&str], out_path: &str) -> (bool, String, String) {
    let mut cmd = assert_cmd::Command::cargo_bin("necom").unwrap();
    cmd.args(args);
    cmd.arg("--outfile").arg(out_path);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (output.status.success(), stdout, stderr)
}

const SENTINEL: &str = "preserve me";

fn assert_outfile_preserved(out_path: &str) {
    let preserved = std::fs::read_to_string(out_path).unwrap();
    assert_eq!(preserved, SENTINEL, "outfile was truncated on failure");
}

// --- Bad infile: simple commands ---

#[test]
fn nwk_stat_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "stat", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_to_dot_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "to-dot", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_topo_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "topo", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_indent_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "indent", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_prune_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "prune", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_reroot_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "reroot", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_to_svg_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "to-svg", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_to_tex_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "to-tex", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_to_forest_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "to-forest", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_distance_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "distance", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_comment_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "comment", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_label_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "label", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_subtree_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "subtree", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_order_bad_infile_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) =
        run_with_outfile(&["nwk", "order", "/nonexistent/path.nwk"], out_path);
    assert!(!success, "expected failure for nonexistent input");
    assert_outfile_preserved(out_path);
}

// --- Arg validation failures with valid infile ---

#[test]
fn nwk_to_svg_bad_width_does_not_truncate_outfile() {
    // Valid infile but invalid --width (zero). Validation should fail before
    // the writer is opened.
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) = run_with_outfile(
        &[
            "nwk",
            "to-svg",
            "tests/newick/catarrhini.nwk",
            "--width",
            "0",
        ],
        out_path,
    );
    assert!(!success, "expected failure for invalid width");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_to_svg_bad_vskip_does_not_truncate_outfile() {
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) = run_with_outfile(
        &[
            "nwk",
            "to-svg",
            "tests/newick/catarrhini.nwk",
            "--vskip",
            "-1",
        ],
        out_path,
    );
    assert!(!success, "expected failure for invalid vskip");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_rename_mismatched_args_does_not_truncate_outfile() {
    // --node count + --lca count must equal --rename count. Here we provide
    // one --node but two --rename values, which should fail validation.
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) = run_with_outfile(
        &[
            "nwk",
            "rename",
            "tests/newick/catarrhini.nwk",
            "--node",
            "Homo",
            "--rename",
            "Human",
            "Extra",
        ],
        out_path,
    );
    assert!(!success, "expected failure for mismatched rename count");
    assert_outfile_preserved(out_path);
}

// --- Multiple file inputs: secondary file failure ---

#[test]
fn nwk_replace_bad_tsv_does_not_truncate_outfile() {
    // Valid infile but nonexistent replace TSV. The TSV load should fail
    // before the writer is opened.
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) = run_with_outfile(
        &[
            "nwk",
            "replace",
            "tests/newick/catarrhini.nwk",
            "/nonexistent/replace.tsv",
        ],
        out_path,
    );
    assert!(!success, "expected failure for nonexistent replace tsv");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_order_bad_name_list_does_not_truncate_outfile() {
    // Valid infile but nonexistent --name-list file.
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, _) = run_with_outfile(
        &[
            "nwk",
            "order",
            "tests/newick/catarrhini.nwk",
            "--name-list",
            "/nonexistent/names.txt",
        ],
        out_path,
    );
    assert!(!success, "expected failure for nonexistent name list");
    assert_outfile_preserved(out_path);
}

#[test]
fn nwk_subtree_empty_condense_does_not_truncate_outfile() {
    // --condense with empty name must fail validation before the writer is
    // opened, so any pre-existing outfile is preserved. The check runs before
    // infile/name-list are loaded, so any infile path suffices.
    let existing = sentinel_outfile(SENTINEL);
    let out_path = existing.path().to_str().unwrap();
    let (success, _, stderr) = run_with_outfile(
        &[
            "nwk",
            "subtree",
            "tests/newick/catarrhini.nwk",
            "--name-list",
            "/nonexistent/names.txt",
            "--condense",
            "",
        ],
        out_path,
    );
    assert!(!success, "expected failure for empty --condense name");
    assert!(
        stderr.contains("--condense requires a non-empty name"),
        "expected helpful error message, got: {stderr}"
    );
    assert_outfile_preserved(out_path);
}
