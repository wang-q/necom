#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::PgrCmd;
use std::path::PathBuf;

/// Return the absolute path to a fixture in `tests/dist/input`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/mat")
        .join(name)
}

#[test]
fn command_dist_vector() {
    let (stdout, _) = PgrCmd::new()
        .args(&[
            "dist",
            "vector",
            fixture("vector.tsv").to_str().unwrap(),
            "--mode",
            "jaccard",
            "--binary",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 16);
    assert!(stdout.contains("A\tA\t1.0000"));
    assert!(stdout.contains("A\tB\t0.3333"));
}
