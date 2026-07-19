#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;
use std::path::PathBuf;

/// Return the absolute path to a fixture in `tests/mat`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/mat")
        .join(name)
}

#[test]
fn command_mat_from_vector() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "from-vector",
            fixture("vector.tsv").to_str().unwrap(),
            "--mode",
            "jaccard",
            "--binary",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 16);
    assert!(stdout.contains("A\tA\t1.000000"));
    assert!(stdout.contains("A\tB\t0.333333"));
}

#[test]
fn command_mat_from_vector_cross_compare() {
    use std::io::Write;
    let mut tmp1 = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp1, "A\t1\t0\t1").unwrap();
    writeln!(tmp1, "B\t0\t1\t1").unwrap();

    let mut tmp2 = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp2, "C\t1\t1\t0").unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "mat",
            "from-vector",
            tmp1.path().to_str().unwrap(),
            tmp2.path().to_str().unwrap(),
            "--mode",
            "jaccard",
            "--binary",
        ])
        .run();

    // Cross-comparison emits one row per pair from set1 x set2 (2 rows).
    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("A\tC\t"));
    assert!(stdout.contains("B\tC\t"));
}

#[test]
fn command_mat_from_vector_length_mismatch() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "A\t1\t2\t3").unwrap();
    writeln!(tmp, "B\t1\t2").unwrap(); // different column count
    let path = tmp.path().to_str().unwrap().to_string();

    let (_, stderr) = NecomCmd::new()
        .args(&["mat", "from-vector", &path, "--mode", "euclid"])
        .run_fail();

    assert!(
        stderr.contains("length mismatch"),
        "expected 'length mismatch' in stderr, got: {}",
        stderr
    );
}

#[test]
fn command_mat_from_vector_parallel_zero() {
    let path_buf = fixture("vector.tsv");
    let path = path_buf.to_str().unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "mat",
            "from-vector",
            path,
            "--mode",
            "jaccard",
            "--binary",
            "-p",
            "0",
        ])
        .run_fail();

    assert!(
        stderr.contains("must be >= 1"),
        "expected '--parallel must be >= 1' in stderr, got: {}",
        stderr
    );
}

#[test]
fn command_mat_from_vector_parallel_two() {
    let path_buf = fixture("vector.tsv");
    let path = path_buf.to_str().unwrap();

    let (p1_out, _) = NecomCmd::new()
        .args(&[
            "mat",
            "from-vector",
            path,
            "--mode",
            "jaccard",
            "--binary",
            "-p",
            "1",
        ])
        .run();

    let (p2_out, _) = NecomCmd::new()
        .args(&[
            "mat",
            "from-vector",
            path,
            "--mode",
            "jaccard",
            "--binary",
            "-p",
            "2",
        ])
        .run();

    assert_eq!(p1_out.lines().count(), 16);
    assert_eq!(p2_out.lines().count(), 16);

    let mut p1_lines: Vec<_> = p1_out.lines().collect();
    let mut p2_lines: Vec<_> = p2_out.lines().collect();
    p1_lines.sort();
    p2_lines.sort();
    assert_eq!(p1_lines, p2_lines);
}

#[test]
fn command_mat_from_vector_bad_outfile() {
    let path_buf = fixture("vector.tsv");
    let path = path_buf.to_str().unwrap();
    let tmpdir = tempfile::tempdir().unwrap();
    let bad_out = tmpdir.path().to_str().unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "mat",
            "from-vector",
            path,
            "--mode",
            "jaccard",
            "--binary",
            "-o",
            bad_out,
        ])
        .run_fail();

    assert!(
        stderr.contains("failed to open writer"),
        "expected writer open error in stderr, got: {}",
        stderr
    );
}
