#[macro_use]
#[path = "common/mod.rs"]
mod common;
use common::NecomCmd;

#[test]
fn command_order_basic() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "order", "tests/newick/abc.nwk", "--num-descendants"])
        .run();

    assert!(stdout.contains("(C,(A,B));"));

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "order",
            "tests/newick/abc.nwk",
            "--num-descendants-rev",
        ])
        .run();

    assert!(stdout.contains("((A,B),C);"));

    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "order", "tests/newick/abc.nwk", "--alphanumeric"])
        .run();

    assert!(stdout.contains("((A,B),C);"));

    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "order", "tests/newick/abc.nwk", "--alphanumeric-rev"])
        .run();

    assert!(stdout.contains("(C,(B,A));"));

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "order",
            "tests/newick/abc.nwk",
            "--alphanumeric-rev",
            "--num-descendants-rev",
        ])
        .run();

    assert!(stdout.contains("((B,A),C);"));
}

#[test]
fn command_order_list() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "order",
            "tests/newick/abcde.nwk",
            "--name-list",
            "tests/newick/abcde.list",
        ])
        .run();

    assert!(stdout.contains("(C:1,(B:1,A:1)D:1)E;"));
}

#[test]
fn command_order_name_list_missing_errors() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut list_file = NamedTempFile::new().unwrap();
    writeln!(list_file, "A").unwrap();
    writeln!(list_file, "Missing1").unwrap();
    writeln!(list_file, "B").unwrap();
    writeln!(list_file, "Missing2").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "nwk",
            "order",
            "tests/newick/abc.nwk",
            "--name-list",
            list_file.path().to_str().unwrap(),
        ])
        .run_fail();

    assert!(
        stderr.contains("name-list entries not found in tree"),
        "expected error for missing names, got stderr: {}",
        stderr
    );
    assert!(
        stderr.contains("Missing1") && stderr.contains("Missing2"),
        "expected missing names in error, got stderr: {}",
        stderr
    );
}

#[test]
fn command_order_unnamed() {
    // Test case where internal nodes are unnamed.
    // ((C,D),(A,B));
    // Without recursive label resolution:
    // (C,D) -> (C,D)
    // (A,B) -> (A,B)
    // Root -> ((C,D),(A,B)) because "" == ""
    //
    // With recursive resolution:
    // (C,D) -> rep "C"
    // (A,B) -> rep "A"
    // Root -> compares "C" vs "A", should be ((A,B),(C,D))

    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "order", "stdin", "--alphanumeric"])
        .stdin("((C,D),(A,B));")
        .run();

    // With --an, it should be sorted
    assert!(stdout.contains("((A,B),(C,D));"));
}

#[test]
fn command_order_species() {
    // Create a temporary directory for testing
    let tempdir = tempfile::tempdir().unwrap();
    let temp_path = tempdir.path();

    std::fs::copy("tests/newick/species.nwk", temp_path.join("species.nwk")).unwrap();

    // Generate a leaf-only list of labels from the tree
    NecomCmd::new()
        .args(&["nwk", "label", "species.nwk", "-I", "-o", "species.list"])
        .current_dir(temp_path)
        .assert()
        .success();

    // Order the tree using the generated list
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "order", "species.nwk", "--name-list", "species.list"])
        .current_dir(temp_path)
        .run();

    // Compare the ordered tree with the original one
    // They should be identical as the list was generated from the original order
    let original = std::fs::read_to_string("tests/newick/species.nwk").unwrap();
    assert_eq!(stdout.trim(), original.trim());

    // gene tree: pmxc.nwk is a proper subtree of species.nwk (missing
    // Brevibac_agri_GCF_004117055_1 and Brevibac_brevis_GCF_900637055_1).
    // Ordering pmxc.nwk with species.list (which contains those missing names)
    // must fail with a clear error rather than silently producing a wrong tree.
    std::fs::copy("tests/newick/pmxc.nwk", temp_path.join("pmxc.nwk")).unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&["nwk", "order", "pmxc.nwk", "--name-list", "species.list"])
        .current_dir(temp_path)
        .run_fail();

    assert!(
        stderr.contains("name-list entries not found in tree"),
        "expected missing-name error for pmxc.nwk, got: {}",
        stderr
    );
    assert!(
        stderr.contains("Brevibac"),
        "expected Brevibac name in error, got: {}",
        stderr
    );
}

#[test]
fn command_order_default_catarrhini() {
    // def:catarrhini.nw
    // Expected: test_nw_order_def.exp
    let expected = "(((Cercopithecus:10,(Macaca:10,Papio:10):20)Cercopithecinae:25,(Colobus:7,Simias:10)Colobinae:5)Cercopithecidae:10,(((Gorilla:16,(Homo:10,Pan:10)Hominini:10)Homininae:15,Pongo:30)Hominidae:15,Hylobates:20):10);";

    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "order", "tests/newick/catarrhini.nwk"])
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn command_order_multiple_trees() {
    // mult:catarrhini_wrong_mult.nw
    // Expected: test_nw_order_mult.exp
    let expected = r#"((((((Cebus,((Cercopithecus,(Macaca,Papio)),Simias)),Hylobates),Pongo),Gorilla),Pan),Homo);
((((((Cebus,((Cercopithecus,(Macaca,Papio)),Simias)),Hylobates),Pongo),Gorilla),Pan),Homo);
((((((Cebus,((Cercopithecus,(Macaca,Papio)),Simias)),Hylobates),Pongo),Gorilla),Pan),Homo);"#;

    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "order", "tests/newick/catarrhini_wrong_mult.nwk"])
        .run();

    // Normalize newlines
    let stdout = stdout.replace("\r\n", "\n");
    assert_eq!(stdout.trim(), expected);
}

#[test]
fn command_order_descendants_tetrapoda() {
    // num: -c n tetrapoda.nw
    // Expected: test_nw_order_num.exp (adjusted for Rust float formatting)
    let expected = "(Tetrao:0.015266,(Bombina:0.269848,(Didelphis:0.007148,((Bradypus:0.020167,(Procavia:0.019702,(Vulpes:0.008083,Orcinus:0.008289)84:0.008124)42:0.003924)16,((Sorex:0.01766,(Mesocricetus:0.011181,Tamias:0.049599)88:0.023597)32:0.000744,(Lepus:0.030777,(Homo:0.004051,(Papio,Hylobates:0.004076)42)99:0.012677)67:0.007717)26:0.006246)78:0.02125)71:0.013125)30:0.006278)100;";

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "order",
            "tests/newick/tetrapoda.nwk",
            "--num-descendants",
        ])
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn command_order_deladderize_verify() {
    // dl: -c d top_heavy_ladder.nw
    // Expected: test_nw_order_dl.exp
    let expected = "(Petromyzon,((Xenopus,((Equus,Homo)Mammalia,Columba)Amniota)Tetrapoda,Carcharodon)Gnathostomata)Vertebrata;";

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "order",
            "tests/newick/top_heavy_ladder.nwk",
            "--deladderize",
        ])
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn command_order_olo_phylip() {
    let expected = "(((B,C),A),D);";

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "order",
            "tests/newick/olo_tree.nwk",
            "--olo",
            "tests/mat/olo_tree.phy",
        ])
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn command_order_olo_pair() {
    let expected = "(((B,C),A),D);";

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "order",
            "tests/newick/olo_tree.nwk",
            "--olo",
            "tests/mat/olo_tree.pair",
            "--olo-format",
            "pair",
        ])
        .run();

    assert_eq!(stdout.trim(), expected);
}

#[test]
fn command_order_olo_missing_leaf_errors() {
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "nwk",
            "order",
            "stdin",
            "--olo",
            "tests/mat/olo_tree.pair",
            "--olo-format",
            "pair",
        ])
        .stdin("(A,B,E);")
        .run_fail();

    assert!(
        stderr.contains("distance matrix missing leaf"),
        "expected missing-leaf error, got stderr: {}",
        stderr
    );
    assert!(
        stderr.contains("E"),
        "expected missing leaf name in error, got stderr: {}",
        stderr
    );
}

#[test]
fn command_order_olo_conflicts_with_other_sorts() {
    let (_, stderr) = NecomCmd::new()
        .args(&[
            "nwk",
            "order",
            "tests/newick/olo_tree.nwk",
            "--olo",
            "tests/mat/olo_tree.phy",
            "--alphanumeric",
        ])
        .run_fail();

    assert!(
        stderr.contains("cannot be used with"),
        "expected conflict error, got stderr: {}",
        stderr
    );
}
