#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;
use std::io::Write;
use tempfile::Builder;

// ================================================================================================
// necom nwk stat
// ================================================================================================

#[test]
fn command_stat_basic() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "stat", "tests/newick/hg38.7way.nwk"])
        .run();

    assert_eq!(stdout.lines().count(), 12);
    assert!(stdout.contains("leaf labels\t7"));
    assert!(stdout.contains("rooted\tYes"));
    assert!(stdout.contains("edges with length\t"));
    assert!(stdout.contains("edges without length\t"));
    assert!(stdout.contains("cherries\t"));
    assert!(stdout.contains("sackin\t"));
    assert!(stdout.contains("colless\t"));
}

#[test]
fn command_stat_catarrhini() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "stat", "tests/newick/catarrhini.nwk"])
        .run();

    assert!(stdout.contains("Type\tphylogram"));
    assert!(stdout.contains("nodes\t19"));
    assert!(stdout.contains("leaves\t10"));
    assert!(stdout.contains("rooted\tYes"));
    assert!(stdout.contains("dichotomies\t9"));
    assert!(stdout.contains("leaf labels\t10"));
    assert!(stdout.contains("internal labels\t6"));
    assert!(stdout.contains("edges with length\t18"));
    assert!(stdout.contains("edges without length\t0"));
    assert!(stdout.contains("cherries\t"));
    assert!(stdout.contains("sackin\t"));
    assert!(stdout.contains("colless\t"));
}

#[test]
fn command_stat_style_line() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "stat",
            "tests/newick/catarrhini.nwk",
            "--style",
            "line",
        ])
        .run();

    assert!(stdout.contains("phylogram\t19\t10\tYes\t9\t10\t6\t18\t0"));
    // Header check
    assert!(stdout.contains(
        "Type\tnodes\tleaves\trooted\tdichotomies\tleaf labels\tinternal labels\tedges with length\tedges without length\tcherries\tsackin\tcolless"
    ));
}

#[test]
fn command_stat_style_short_flag() {
    let (stdout_long, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "stat",
            "tests/newick/catarrhini.nwk",
            "--style",
            "line",
        ])
        .run();
    let (stdout_short, _) = NecomCmd::new()
        .args(&["nwk", "stat", "tests/newick/catarrhini.nwk", "-s", "line"])
        .run();
    assert_eq!(stdout_long, stdout_short);
    assert!(stdout_short.contains("phylogram\t19\t10\tYes\t9\t10\t6\t18\t0"));
}

#[test]
fn command_stat_forest() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "stat", "tests/newick/forest.nwk", "--style", "line"])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 6);

    // Header
    assert!(lines[0].contains(
        "Type\tnodes\tleaves\trooted\tdichotomies\tleaf labels\tinternal labels\tedges with length\tedges without length\tcherries\tsackin\tcolless"
    ));

    // Tree 1: Cladogram, 18 nodes, 11 leaves, No rooted, 5 dichotomies, 11 leaf labels, 0 inner labels, 0 edges with length, 17 without
    assert!(lines[1].contains("cladogram\t18\t11\tNo\t5\t11\t0\t0\t17"));

    // Tree 2: Cladogram, 13 nodes, 8 leaves, No rooted, 3 dichotomies, 8 leaf labels, 0 inner labels, 0 edges with length, 12 without
    assert!(lines[2].contains("cladogram\t13\t8\tNo\t3\t8\t0\t0\t12"));

    // Tree 3: Phylogram, 10 nodes, 6 leaves, No rooted, 3 dichotomies, 6 leaf labels, 0 inner labels, 9 edges with length, 0 without
    assert!(lines[3].contains("phylogram\t10\t6\tNo\t3\t6\t0\t9\t0"));

    // Tree 4: Phylogram, 19 nodes, 10 leaves, 9 dichotomies, 10 leaf labels, 6 inner labels, 18 edges with length, 0 without
    assert!(lines[4].contains("phylogram\t19\t10\tYes\t9\t10\t6\t18\t0"));

    // Tree 5: Cladogram, 19 nodes, 10 leaves, 9 dichotomies, 10 leaf labels, 0 inner labels, 0 edges with length, 18 without
    assert!(lines[5].contains("cladogram\t19\t10\tYes\t9\t10\t0\t0\t18"));
}

#[test]
fn command_stat_stdin() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "stat", "stdin"])
        .stdin("((A:1,B:1):1,C:2);")
        .run();

    assert!(stdout.contains("nodes\t5"));
    assert!(stdout.contains("leaves\t3"));
    assert!(stdout.contains("leaf labels\t3"));
}

#[test]
fn command_stat_multi_tree_stdin() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "stat", "stdin"])
        .stdin("(A,B)C;(D,E)F;")
        .run();

    // Should appear twice (once for each tree)
    assert_eq!(stdout.matches("nodes\t3").count(), 2);
    assert_eq!(stdout.matches("leaves\t2").count(), 2);
}

#[test]
fn command_stat_outfile() {
    let temp_file = Builder::new().suffix(".tsv").tempfile().unwrap();
    let outfile = temp_file.path().to_str().unwrap();

    NecomCmd::new()
        .args(&["nwk", "stat", "tests/newick/catarrhini.nwk", "-o", outfile])
        .assert()
        .success();

    let content = std::fs::read_to_string(outfile).unwrap();
    assert!(content.contains("nodes\t19"));
    assert!(content.contains("leaves\t10"));
}

// ================================================================================================
// necom nwk label
// ================================================================================================

#[test]
fn command_label_basic() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "label", "tests/newick/hg38.7way.nwk"])
        .run();

    // hg38.7way.nwk has 7 leaves (Human, Chimp, Rhesus, Mouse, Rat, Dog, Opossum)
    // and presumably no named internal nodes.
    assert_eq!(stdout.lines().count(), 7);
    assert!(stdout.contains("Human\n"));
}

#[test]
fn command_label_leaf_only() {
    // -I: Don't print internal labels (so print leaves only)
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "label", "tests/newick/catarrhini.nwk", "-I"])
        .run();

    // catarrhini.nwk has 10 leaves and 6 internal labels
    assert_eq!(stdout.lines().count(), 10);
    assert!(stdout.contains("Homo"));
    assert!(!stdout.contains("Hominini")); // Hominini is internal
}

#[test]
fn command_label_internal_only() {
    // -L: Don't print leaf labels (so print internal only)
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "label", "tests/newick/catarrhini.nwk", "-L"])
        .run();

    assert_eq!(stdout.lines().count(), 6);
    assert!(stdout.contains("Hominini"));
    assert!(!stdout.contains("Homo"));
}

#[test]
fn command_label_empty_internal() {
    // Test on a tree with no internal labels using -L
    // hg38.7way.nwk has no internal labels
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "label", "tests/newick/hg38.7way.nwk", "-L"])
        .run();

    assert_eq!(stdout.lines().count(), 0);
}

#[test]
fn command_label_selection_node_monophyly() {
    // -n selection with -M (monophyly) and -D (descendants)
    // -n Homininae -n Pongo
    // In catarrhini.nwk, Homininae is an internal node. Pongo is a leaf (genus).
    // -D includes descendants.
    // -M checks monophyly.
    // Without -I, internal labels are preserved.
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "label",
            "tests/newick/catarrhini.nwk",
            "-n",
            "Homininae",
            "-n",
            "Pongo",
            "-DM",
        ])
        .run();

    // Homininae + its descendants (Gorilla, Pan, Homo, Hominini) + Pongo = 6.
    // All belong to the Hominidae clade, so -M passes.
    assert_eq!(stdout.lines().count(), 6);
    assert!(stdout.contains("Homininae"));
    assert!(stdout.contains("Hominini"));
    assert!(stdout.contains("Gorilla"));
    assert!(stdout.contains("Pan"));
    assert!(stdout.contains("Homo"));
    assert!(stdout.contains("Pongo"));
}

#[test]
fn command_label_monophyly_respects_internal_flag() {
    // -M no longer overrides -I: selecting an internal node with -D and -M
    // but also -I should output only the leaf labels of that clade.
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "label",
            "tests/newick/catarrhini.nwk",
            "-n",
            "Hominidae",
            "-DMI",
        ])
        .run();

    // Hominidae's descendants: Gorilla, Pan, Homo, Pongo.
    assert_eq!(stdout.lines().count(), 4);
    assert!(stdout.contains("Gorilla"));
    assert!(stdout.contains("Pan"));
    assert!(stdout.contains("Homo"));
    assert!(stdout.contains("Pongo"));
    assert!(!stdout.contains("Hominidae"));
}

#[test]
fn command_label_selection_file() {
    // -l name-list file input
    let mut temp_file = Builder::new().suffix(".txt").tempfile().unwrap();
    writeln!(temp_file, "Homo").unwrap();
    writeln!(temp_file, "Pan").unwrap();
    let list_file = temp_file.path().to_str().unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "label",
            "tests/newick/catarrhini.nwk",
            "-l",
            list_file,
        ])
        .run();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("Homo"));
    assert!(stdout.contains("Pan"));
}

#[test]
fn command_label_regex() {
    // -r regex
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "label",
            "tests/newick/hg38.7way.nwk",
            "-x",
            "^ch", // Case insensitive by default?
        ])
        .run();

    // Should match Chimp
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("Chimp"));
}

#[test]
fn command_label_regex_case_insensitive() {
    // Verify case insensitivity explicitly
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "label",
            "tests/newick/catarrhini.nwk",
            "-x",
            "^homo", // lowercase
        ])
        .run();

    // Should match Homo
    // But NOT Hominoidea (starts with Homi, not Homo)
    assert!(stdout.contains("Homo"));
    assert!(!stdout.contains("Hominoidea"));
}

#[test]
fn command_label_columns() {
    // -c columns
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "label",
            "tests/newick/catarrhini.comment.nwk",
            "-c",
            "species",
        ])
        .run();

    // Output format: Label \t Species
    // Example: Homo \t Homo
    // We expect a tab
    assert!(stdout.contains("\tHomo\n"));
}

#[test]
fn command_label_columns_full() {
    // -c full should emit the comment in NHX format
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "label",
            "tests/newick/catarrhini.comment.nwk",
            "-c",
            "full",
        ])
        .run();

    assert!(stdout.contains("Homo\t[&&NHX:S=Homo]\n"));
    assert!(stdout.contains("Gorilla\t[&&NHX:S=Gorilla]\n"));
}

#[test]
fn command_label_formatting_root() {
    // --root
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "label", "tests/newick/root.nwk", "--root"])
        .run();

    assert!(stdout.trim().contains("Root"));
    assert_eq!(stdout.lines().count(), 1);
}

#[test]
fn command_label_formatting_tab() {
    // --tab (tab separated on one line)
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "label", "tests/newick/catarrhini.nwk", "--tab"])
        .run();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("Homo"));
    assert!(stdout.contains('\t'));
}

#[test]
fn command_label_short_flags_and_aliases() {
    let (stdout_long, _) = NecomCmd::new()
        .args(&["nwk", "label", "tests/newick/catarrhini.nwk", "--tab"])
        .run();
    let (stdout_short, _) = NecomCmd::new()
        .args(&["nwk", "label", "tests/newick/catarrhini.nwk", "-t"])
        .run();
    assert_eq!(stdout_long, stdout_short);

    let (stdout_long, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "label",
            "tests/newick/catarrhini.comment.nwk",
            "--extra-column",
            "species",
        ])
        .run();
    let (stdout_alias, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "label",
            "tests/newick/catarrhini.comment.nwk",
            "--column",
            "species",
        ])
        .run();
    assert_eq!(stdout_long, stdout_alias);
    assert!(stdout_alias.contains("\tHomo\n"));
}

#[test]
fn command_label_special_chars() {
    // Special chars (slash, space)
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "label", "tests/newick/slash_and_space.nwk"])
        .run();

    assert!(stdout.contains("B/Washington/05/2009 gi_255529494 gb_GQ451489\n"));
    assert!(stdout.contains("Swit/1562056/2009_NA\n"));
    assert!(stdout.lines().count() > 10);
}

#[test]
fn command_label_multi_tree() {
    // Multiple trees in one file, --tab option
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "label", "tests/newick/forest.nwk", "--tab"])
        .run();

    // forest.nwk has 5 trees, so 5 lines
    assert_eq!(stdout.lines().count(), 5);
    assert!(stdout.contains("Pandion")); // Tree 1
    assert!(stdout.contains("Diomedea")); // Tree 2
    assert!(stdout.contains("Ticodendraceae")); // Tree 3
    assert!(stdout.contains("Gorilla")); // Tree 4/5
}

#[test]
fn command_label_single_node_monophyly() {
    // A single selected internal node should not pass the -M monophyly check.
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "label", "stdin", "-n", "X", "-M"])
        .stdin("((A,B)X,(C,D)Y);")
        .run();

    assert!(stdout.trim().is_empty());
}

// ================================================================================================
// necom nwk distance
// ================================================================================================

#[test]
fn command_distance_root() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "-I",
            "--mode",
            "root",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 10);
    assert!(stdout.contains("Homo\t60"));
}

#[test]
fn command_distance_parent() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "-I",
            "--mode",
            "parent",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 10);
    assert!(stdout.contains("Homo\t10"));
}

#[test]
fn command_distance_pairwise() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "-I",
            "--mode",
            "pairwise",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 100);
    assert!(stdout.contains("Homo\tPongo\t65"));
    assert!(stdout.contains("Pongo\tHomo\t65"));
}

#[test]
fn command_distance_lca() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "-I",
            "--mode",
            "lca",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 100);
    assert!(stdout.contains("Homo\tPongo\t35\t30"));
    assert!(stdout.contains("Homo\tHomo\t0\t0"));
}

#[test]
fn command_distance_phylip() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "-I",
            "--mode",
            "phylip",
        ])
        .run();

    assert!(stdout.lines().count() >= 11);
    assert!(stdout.trim().starts_with("10"));
    assert!(stdout.contains("Homo"));
    assert!(stdout.contains(" 65.000000"));
}

#[test]
fn command_distance_mode_short_flag() {
    let (stdout_long, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "-I",
            "--mode",
            "root",
        ])
        .run();
    let (stdout_short, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "-I",
            "-m",
            "root",
        ])
        .run();
    assert_eq!(stdout_long, stdout_short);
    assert!(stdout_short.contains("Homo\t60"));
}

#[test]
fn command_distance_node_selection() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "--mode",
            "root",
            "-n",
            "Homo",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("Homo\t60"));
}

#[test]
fn command_distance_node_selection_multiple() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "--mode",
            "root",
            "-n",
            "Homo",
            "-n",
            "Pan",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("Homo\t60"));
    assert!(stdout.contains("Pan\t60"));
}

#[test]
fn command_distance_name_list_selection() {
    let mut temp_file = Builder::new().suffix(".txt").tempfile().unwrap();
    writeln!(temp_file, "Homo").unwrap();
    writeln!(temp_file, "Pan").unwrap();
    let list_file = temp_file.path().to_str().unwrap();

    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "--mode",
            "root",
            "-l",
            list_file,
        ])
        .run();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("Homo\t60"));
    assert!(stdout.contains("Pan\t60"));
}

#[test]
fn command_distance_regex_selection() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "--mode",
            "root",
            "-x",
            "^Homo$",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("Homo\t60"));
    assert!(!stdout.contains("Hominini"));
}

#[test]
fn command_distance_selection_with_internal_filter() {
    // -n selects an internal node; -I filters it out, so output is empty.
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/catarrhini.nwk",
            "--mode",
            "root",
            "-n",
            "Hominidae",
            "-I",
        ])
        .run();

    assert!(stdout.trim().is_empty());
}

#[test]
fn command_distance_stdin() {
    // Topological distance (stdin input)
    let input = "((A,B)C,D)E;";
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "distance", "stdin", "--mode", "root"])
        .stdin(input)
        .run();

    assert!(stdout.contains("A\t2"));
    assert!(stdout.contains("B\t2"));
    assert!(stdout.contains("C\t1"));
    assert!(stdout.contains("D\t1"));
    assert!(stdout.contains("E\t0"));
}

#[test]
fn command_distance_reference_dist_root() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "distance", "tests/newick/dist.nwk", "--mode", "root"])
        .run();

    // Verified against newick_utils test_nw_distance_rh.exp
    assert!(stdout.contains("A\t4"));
    assert!(stdout.contains("B\t6"));
    assert!(stdout.contains("C\t3"));
    assert!(stdout.contains("D\t6"));
    assert!(stdout.contains("E\t4"));
    assert!(stdout.contains("F\t4"));
}

#[test]
fn command_distance_reference_unnamed() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/dist_meth_xpl.nwk",
            "--mode",
            "root",
        ])
        .run();

    // Verified against newick_utils test_nw_distance_nsf.exp
    // Unnamed nodes receive synthetic labels of the form "#<id>" (e.g., "#2", "#3").
    assert!(stdout.contains("#2\t3"));
    assert!(stdout.contains("#3\t4"));
    assert!(stdout.contains("A\t5"));
    assert!(stdout.contains("B\t4"));
}

#[test]
fn command_distance_reference_lca() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "distance", "tests/newick/dist.nwk", "--mode", "lca"])
        .run();

    // Verified against newick_utils test_nw_distance_an_2.exp (D F -> 4 2)
    // necom output: D \t F \t 4 \t 2
    assert!(stdout.contains("D\tF\t4\t2"));
}

#[test]
fn command_distance_reference_pairwise() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/dist.nwk",
            "--mode",
            "pairwise",
        ])
        .run();

    // Verified against newick_utils test_nw_distance_pan.exp (F D E B)
    // Check F-D distance (2+4=6)
    // Check F-E distance (2+2=4)
    // Check D-B distance (6+6=12)
    assert!(stdout.contains("F\tD\t6"));
    assert!(stdout.contains("F\tE\t4"));
    assert!(stdout.contains("D\tB\t12"));
}

#[test]
fn command_distance_reference_phylip() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/dist.nwk",
            "--mode",
            "phylip",
        ])
        .run();

    // Verified against newick_utils test_nw_distance_m.exp
    // Note: necom includes all named nodes (leaves + internal) in phylip mode.
    // dist.nwk has 6 leaves + 5 named internal nodes = 11 nodes.
    // newick_utils defaults to leaves only for matrix.
    assert!(stdout.lines().next().unwrap().trim().starts_with("11"));
    assert!(stdout.contains("A"));
    assert!(stdout.contains("B"));
    // Check for some distance values in the output
    assert!(stdout.contains("6.000000"));
    assert!(stdout.contains("7.000000"));
    assert!(stdout.contains("10.000000"));
}

#[test]
fn command_distance_float_noise() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/noise.nwk",
            "--mode",
            "pairwise",
        ])
        .run();

    // A->B should be 0.1 + 0.2 = 0.3
    // Without fix: 0.30000000000000004
    // With fix: 0.3
    assert!(stdout.contains("A\tB\t0.3\n") || stdout.contains("A\tB\t0.30\n")); // Allow formatted but clean
    assert!(!stdout.contains("0.30000000000000004"));
}

#[test]
fn command_distance_reference_parent() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            "tests/newick/dist.nwk",
            "--mode",
            "parent",
        ])
        .run();

    // Verified against newick_utils test_nw_distance_par_all_nam.exp
    // A->g: 2, B->g: 4, g->k: 2
    // C->j: 2, D->h: 3, E->h: 1
    // h->i: 1, F->i: 2, i->j: 1
    // j->k: 1, k->None: 0
    assert!(stdout.contains("A\t2"));
    assert!(stdout.contains("B\t4"));
    assert!(stdout.contains("g\t2"));
    assert!(stdout.contains("C\t2"));
    assert!(stdout.contains("D\t3"));
    assert!(stdout.contains("E\t1"));
    assert!(stdout.contains("h\t1"));
    assert!(stdout.contains("F\t2"));
    assert!(stdout.contains("i\t1"));
    assert!(stdout.contains("j\t1"));
    assert!(stdout.contains("k\t0"));
}

#[test]
fn command_distance_phylip_rejects_unnamed_nodes() {
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file, "((A,B),(C,D))Root;").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            file.path().to_str().unwrap(),
            "--mode",
            "phylip",
        ])
        .run_fail();

    assert!(
        stderr.contains("Phylip matrix requires all selected nodes to be named"),
        "expected clear error for unnamed nodes, got stderr: {}",
        stderr
    );
}

#[test]
fn command_distance_phylip_rejects_whitespace_in_names() {
    let mut file = Builder::new().suffix(".nwk").tempfile().unwrap();
    writeln!(file, "('A B',C);").unwrap();

    let (_, stderr) = NecomCmd::new()
        .args(&[
            "nwk",
            "distance",
            file.path().to_str().unwrap(),
            "--mode",
            "phylip",
            "-n",
            "A B",
        ])
        .run_fail();

    assert!(
        stderr.contains("Phylip matrix requires node names without whitespace"),
        "expected clear error for whitespace in names, got stderr: {}",
        stderr
    );
}

// ================================================================================================
// Edge cases
// ==============================================================================================

#[test]
fn command_stat_empty_input() {
    let (_, stderr) = NecomCmd::new()
        .args(&["nwk", "stat", "stdin"])
        .stdin("")
        .run_fail();

    assert!(!stderr.is_empty());
}

#[test]
fn command_indent_single_node() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "stdin", "--compact"])
        .stdin("A;")
        .run();

    assert_eq!(stdout.trim(), "A;");
}

#[test]
fn command_indent_non_finite_lengths_omitted() {
    // NaN, negative, and infinite branch lengths are omitted on output.
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "stdin", "--compact"])
        .stdin("(A:-1,B:NaN)Root;")
        .run();

    assert_eq!(stdout.trim(), "(A,B)Root;");
}

#[test]
fn command_indent_zero_length_omitted() {
    // Zero branch lengths are also omitted to keep cladograms clean.
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "stdin", "--compact"])
        .stdin("(A:0,B:0)Root;")
        .run();

    assert_eq!(stdout.trim(), "(A,B)Root;");
}

#[test]
fn command_indent_text_short_flag() {
    let input = "(A,B)Root;";
    let (stdout_long, _) = NecomCmd::new()
        .args(&["nwk", "indent", "stdin", "--text", "."])
        .stdin(input)
        .run();
    let (stdout_short, _) = NecomCmd::new()
        .args(&["nwk", "indent", "stdin", "-t", "."])
        .stdin(input)
        .run();
    assert_eq!(stdout_long, stdout_short);
    assert!(stdout_short.contains("\n."));
}
