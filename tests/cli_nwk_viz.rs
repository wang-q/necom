#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;

#[test]
fn command_indent() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "indent",
            "tests/newick/hg38.7way.nwk",
            "--text",
            ".   ",
        ])
        .run();

    assert_eq!(stdout.lines().count(), 19);
    assert!(stdout.contains(".   .   Human:"));
    assert!(stdout.contains("\n.   Opossum:"));
}

#[test]
fn command_indent_compact() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "tests/newick/catarrhini.nwk", "--compact"])
        .run();

    assert_eq!(stdout.lines().count(), 1);
    assert_eq!(stdout.trim().lines().count(), 1); // Ensure only one line after trim
    assert!(stdout.contains("Gorilla"));
}

#[test]
fn command_indent_simple() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "tests/newick/catarrhini_wrong.nwk"])
        .run();

    assert!(stdout.contains("  Homo,"));
    assert!(stdout.contains("      Gorilla,"));
    assert_eq!(stdout.lines().count(), 28);
}

#[test]
fn command_indent_optt() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "indent",
            "tests/newick/catarrhini_wrong.nwk",
            "--text",
            ".  ",
        ])
        .run();

    assert!(stdout.contains(".  Homo,"));
    assert!(stdout.contains(".  .  .  Gorilla,"));
    assert_eq!(stdout.lines().count(), 28);
}

#[test]
fn command_indent_multiple_optc() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "tests/newick/forest_ind.nwk", "--compact"])
        .run();

    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 5);
    assert!(lines[0].starts_with("(Pandion,"));
    assert!(lines[4].starts_with("(Homo,"));
}

#[test]
fn command_indent_stdin() {
    // 1. Default indentation
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "stdin"])
        .stdin("((A,B),C);")
        .run();

    // Should have newlines and spaces (default 2 spaces)
    assert!(stdout.contains("  A"));
    assert!(stdout.contains("  B"));
    assert!(stdout.contains("C"));
}

#[test]
fn command_indent_special_chars() {
    // 1. Plus/Minus in labels (plusminus.nw)
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "tests/newick/plusminus.nwk"])
        .run();

    // necom should output it, likely quoted if it contains special chars that require quoting.
    // + is not strictly a special char in Newick (unlike (),:;), but some parsers might quote it.
    // necom quote_label: "(),:;[] \t\n".contains(c) -> quotes.
    // + is NOT in that list. So it should be unquoted.
    assert!(stdout.contains("HRV-A+A2"));

    // 2. Slash and Space (slash_and_space.nw)
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "tests/newick/slash_and_space.nwk"])
        .run();

    // Label: B/Washington/05/2009 gi_255529494 gb_GQ451489
    // Contains space, so necom WILL quote it.
    // newick_utils might not quote it if it's lax, but necom is safer.
    // We just check if the text is present.
    assert!(stdout.contains("B/Washington/05/2009 gi_255529494 gb_GQ451489"));
    // Check if it is quoted
    assert!(stdout.contains("'B/Washington/05/2009 gi_255529494 gb_GQ451489'"));
}

#[test]
fn command_indent_multiple_trees() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "tests/newick/forest.nwk"])
        .run();

    // forest.nwk contains multiple trees (5 lines).
    // necom should output all of them.
    // Verify specific labels from different trees to ensure all are processed.
    assert!(stdout.contains("Pandion")); // From tree 1
    assert!(stdout.contains("Diomedea")); // From tree 2
    assert!(stdout.contains("Ticodendraceae")); // From tree 3
    assert!(stdout.contains("Gorilla")); // From tree 4
    assert!(stdout.contains("Cebus")); // From tree 5

    // Verify we have at least 5 semicolons (one per tree)
    assert!(stdout.matches(';').count() >= 5);
}

#[test]
fn command_comment() {
    // This test involves piping output from one necom command to another.
    // 1. Run necom nwk comment ... --color green
    let (color_stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "comment",
            "tests/newick/abc.nwk",
            "-n",
            "A",
            "-n",
            "C",
            "--color",
            "green",
        ])
        .run();

    // 2. Run necom nwk comment stdin ... --dot with input from step 1
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "comment", "stdin", "-l", "A,B", "--dot"])
        .stdin(color_stdout)
        .run();

    assert_eq!(
        stdout.trim(),
        "((A[&&NHX:color=green],B)[&&NHX:dot=black],C[&&NHX:color=green]);"
    );
}

#[test]
fn command_comment_remove() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "comment",
            "tests/newick/abc.comment.nwk",
            "--remove",
            "color=",
        ])
        .run();

    assert_eq!(stdout.trim(), "((A,B)[&&NHX:dot=black],C);");
}

#[test]
fn command_comment_string_free_form() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "comment",
            "tests/newick/abc.nwk",
            "-n",
            "A",
            "--string",
            "hello world",
        ])
        .run();

    assert_eq!(stdout.trim(), "((A[&&NHX:string=hello world],B),C);");
}

#[test]
fn command_to_dot() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-dot", "tests/newick/catarrhini.nwk"])
        .run();

    assert!(stdout.contains("digraph Tree {"));
    assert!(stdout.contains("node [shape=box];"));
    assert!(stdout.contains("Hominidae"));
    assert!(stdout.contains("->"));
}

#[test]
fn command_to_forest() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-forest", "tests/newick/catarrhini.nwk"])
        .run();

    assert!(!stdout.contains(",,"));
    assert!(stdout.contains("Hominidae"));
    assert!(stdout.contains("{Homo}"));
    assert!(stdout.contains("[, tier="));
}

#[test]
fn command_to_forest_bl() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-forest", "tests/newick/catarrhini.nwk", "--bl"])
        .run();

    assert!(stdout.contains("l=")); // Should have lengths
    assert!(stdout.contains("Hominidae"));
    assert!(stdout.contains("{Homo}"));
}

#[test]
fn command_tex() {
    // 1. Default (Cladogram)
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-tex", "tests/newick/hg38.7way.nwk"])
        .run();

    assert!(stdout.contains(r"\documentclass"));
    assert!(stdout.contains(r"\begin{forest}"));
    assert!(stdout.contains("tier=4"));

    // 2. Phylogram (--bl)
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-tex", "tests/newick/hg38.7way.nwk", "--bl"])
        .run();

    assert!(stdout.contains(r"\documentclass"));
    assert!(stdout.contains("l=40mm"));
    assert!(stdout.contains("l=53mm"));
}

#[test]
fn command_to_svg() {
    // catarrhini.nwk has branch lengths, so it should auto-detect phylogram mode
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-svg", "tests/newick/catarrhini.nwk"])
        .run();

    assert!(stdout.contains("<?xml version=\"1.0\""));
    assert!(stdout.contains("<svg xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(stdout.contains("<style>"));
    assert!(stdout.contains("Hominidae"));
    assert!(stdout.contains("Homo"));
    assert!(stdout.contains("class=\"dot\""));
    // Scale bar should be present (auto-detected phylogram)
    assert!(stdout.contains("class=\"scale-text\""));
}

#[test]
fn command_to_svg_cladogram() {
    // abc.nwk has no branch lengths, so it should auto-detect cladogram mode
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-svg", "tests/newick/abc.nwk"])
        .run();

    assert!(stdout.contains("<svg xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(stdout.contains("A"));
    assert!(stdout.contains("B"));
    assert!(stdout.contains("C"));
    // No scale bar in cladogram mode
    assert!(!stdout.contains("class=\"scale-text\""));
}

#[test]
fn command_to_svg_width_vskip() {
    // Custom width and vskip
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "to-svg",
            "tests/newick/abc.nwk",
            "-w",
            "400",
            "-v",
            "30",
        ])
        .run();

    assert!(stdout.contains("<svg xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(stdout.contains("A"));
}

#[test]
fn command_viz_escapes_special_chars() {
    // Leaf label contains LaTeX/XML/DOT special characters.
    let newick = "(A,'A{B}C\\D'[&&NHX:label=E%F:comment=G&H])Root;";

    // to-forest must escape LaTeX special characters.
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-forest", "stdin"])
        .stdin(newick)
        .run();
    assert!(
        stdout.contains(r"\{"),
        "expected escaped {{ in forest: {}",
        stdout
    );
    assert!(
        stdout.contains(r"\}"),
        "expected escaped }} in forest: {}",
        stdout
    );
    assert!(
        stdout.contains(r"\textbackslash{}"),
        "expected escaped backslash in forest: {}",
        stdout
    );
    assert!(
        stdout.contains(r"\%"),
        "expected escaped % in forest: {}",
        stdout
    );
    assert!(
        stdout.contains(r"\&"),
        "expected escaped & in forest: {}",
        stdout
    );
    assert!(
        !stdout.contains("{B}"),
        "unescaped brace group in forest: {}",
        stdout
    );
    assert!(
        !stdout.contains(r"C\D"),
        "unescaped backslash in forest: {}",
        stdout
    );
    assert!(!stdout.contains("E%F"), "unescaped % in forest: {}", stdout);
    assert!(!stdout.contains("G&H"), "unescaped & in forest: {}", stdout);

    // to-dot must escape backslashes and quotes.
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-dot", "stdin"])
        .stdin(newick)
        .run();
    assert!(stdout.contains(r#"label="A{B}C\\D""#));

    // to-svg leaves braces and backslashes as-is (only XML specials are escaped).
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-svg", "stdin"])
        .stdin(newick)
        .run();
    assert!(stdout.contains("A{B}C\\D"));

    // indent round-trip preserves the original label (quotes are optional).
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "indent", "stdin"])
        .stdin(newick)
        .run();
    assert!(stdout.contains("A{B}C\\D"));
}

#[test]
fn command_to_tex_bl_scale_bar() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-tex", "tests/newick/hg38.7way.nwk", "--bl"])
        .run();

    assert!(stdout.contains(r"\draw"));
    assert!(stdout.contains(r"\scriptsize{"));
    assert!(!stdout.contains("NaN"));
    assert!(!stdout.contains("-0"));
}

#[test]
fn command_to_tex_forest_pass_through() {
    let forest = "[A[B][C]]";
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-tex", "stdin", "--forest"])
        .stdin(forest)
        .run();

    assert!(stdout.contains(r"\begin{forest}"));
    assert!(stdout.contains("[A[B][C]]"));
    assert!(stdout.contains(r"\end{forest}"));
}

#[test]
fn command_to_tex_forest_pass_through_does_not_shadow_style_markers() {
    // User-provided Forest code containing the style marker substring should not
    // break the template's style replacement.
    let forest = "[A[B][C]] %STYLE_BEGIN";
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-tex", "stdin", "--forest", "--no-default-style"])
        .stdin(forest)
        .run();

    assert!(stdout.contains(r"\begin{forest}"));
    assert!(stdout.contains("[A[B][C]] %STYLE_BEGIN"));
    assert!(stdout.contains(r"\end{forest}"));
    // The template's original font setup must still be present; if the style
    // replacement had been confused by the user Forest code, it would be lost.
    assert!(stdout.contains("Fira Sans"));
}

#[test]
fn command_to_tex_no_default_style() {
    let (stdout, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "to-tex",
            "tests/newick/hg38.7way.nwk",
            "--no-default-style",
        ])
        .run();

    assert!(stdout.contains("Fira Sans"));
    assert!(!stdout.contains("NotoSans"));
}

#[test]
fn command_to_tex_style_short_flag_and_alias() {
    let (stdout_long, _) = NecomCmd::new()
        .args(&[
            "nwk",
            "to-tex",
            "tests/newick/hg38.7way.nwk",
            "--no-default-style",
        ])
        .run();
    let (stdout_short, _) = NecomCmd::new()
        .args(&["nwk", "to-tex", "tests/newick/hg38.7way.nwk", "-s"])
        .run();
    assert_eq!(stdout_long, stdout_short);
    assert!(stdout_short.contains("Fira Sans"));
    assert!(!stdout_short.contains("NotoSans"));

    let (stdout_alias, _) = NecomCmd::new()
        .args(&["nwk", "to-tex", "tests/newick/hg38.7way.nwk", "--style"])
        .run();
    assert_eq!(stdout_long, stdout_alias);
}

#[test]
fn command_to_forest_color_latex_escaped() {
    // Color values containing LaTeX special characters must be escaped.
    let newick = "(A[&&NHX:color=#FF0000],B)Root;";
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-forest", "stdin"])
        .stdin(newick)
        .run();

    assert!(
        stdout.contains(r"\#"),
        "expected escaped # in forest color: {}",
        stdout
    );
    assert!(
        !stdout.contains("color={#FF0000}"),
        "unescaped # in forest color: {}",
        stdout
    );
}

#[test]
fn command_to_dot_escapes_newline() {
    // Real newline characters in labels must be escaped for DOT.
    let newick = "('A\nB',C)Root;";
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-dot", "stdin"])
        .stdin(newick)
        .run();

    // The DOT label should contain the two-character escape sequence \n,
    // not an actual newline.
    assert!(
        stdout.contains("label=\"A\\nB\""),
        "expected escaped newline in DOT label: {}",
        stdout
    );
    assert!(
        !stdout.contains("label=\"A\nB\""),
        "real newline in DOT label: {}",
        stdout
    );
}

#[test]
fn command_to_tex_bl_tiny_branch_lengths() {
    let (stdout, _) = NecomCmd::new()
        .args(&["nwk", "to-tex", "stdin", "--bl"])
        .stdin("(A:1e-10,B:2e-10)R;")
        .run();

    assert!(stdout.contains(r"\draw"));
    assert!(!stdout.contains("NaN"));
    assert!(!stdout.contains("-0"));
}
