use anyhow::Context;
use clap::{Arg, ArgAction, ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
use std::collections::BTreeSet;
use std::io::{Read, Write};

/// Build the clap subcommand for compare.
pub fn make_subcommand() -> Command {
    Command::new("compare")
        .about("Compares trees (RF, WRF, KF distances)")
        .after_help(include_str!("../../../docs/help/eval/compare.md"))
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "First input filename (or stdin)",
        ))
        .arg(
            Arg::new("compare_file")
                .index(2)
                .num_args(1)
                .help("Second input filename (or stdin)"),
        )
        .arg(
            Arg::new("include_trivial")
                .long("include-trivial")
                .action(ArgAction::SetTrue)
                .help("Include trivial splits (single-leaf branches) in WRF/KF"),
        )
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Load trees from `infile`, bailing with a clear message when the input is
/// empty or contains no parseable trees.
fn load_trees(infile: &str) -> anyhow::Result<Vec<Tree>> {
    let mut reader =
        necom::reader(infile).with_context(|| format!("failed to open {}", infile))?;
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .with_context(|| format!("failed to read {}", infile))?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    Tree::from_newick_multi(&content)
        .with_context(|| format!("failed to parse '{}'", infile))
}

/// Build a sorted set of leaf names for `tree`, bailing on unnamed or
/// duplicate leaves. Used by [`validate_leaf_consistency`] to pre-check leaf
/// sets before the writer is opened, so that mismatches do not leave a
/// partial output file behind.
fn canonical_leaf_set(tree: &Tree) -> anyhow::Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for name in tree.get_leaf_names() {
        let name =
            name.ok_or_else(|| anyhow::anyhow!("tree contains an unnamed leaf"))?;
        if !names.insert(name.clone()) {
            anyhow::bail!("duplicate leaf name: {}", name);
        }
    }
    Ok(names)
}

/// Validate that all trees in `trees` share the same leaf set.
///
/// `compute_tree_metrics` requires exact leaf-set equality for every pair.
/// Running this check upfront (before the writer is opened) avoids writing a
/// partial output file (header + some rows) when a later pair fails.
fn validate_leaf_consistency(trees: &[Tree], label: &str) -> anyhow::Result<()> {
    if trees.len() < 2 {
        return Ok(());
    }
    let first = canonical_leaf_set(&trees[0])
        .with_context(|| format!("invalid leaves in tree 1 of {}", label))?;
    for (i, t) in trees.iter().enumerate().skip(1) {
        let here = canonical_leaf_set(t)
            .with_context(|| format!("invalid leaves in tree {} of {}", i + 1, label))?;
        if here != first {
            let only_here: Vec<_> = here.difference(&first).collect();
            let only_first: Vec<_> = first.difference(&here).collect();
            anyhow::bail!(
                "tree {} in {} has a different leaf set from tree 1: only here {:?}, only in first {:?}",
                i + 1,
                label,
                only_here,
                only_first
            );
        }
    }
    Ok(())
}

/// Execute the compare command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // 1. Load first file
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let compare_file = args.get_one::<String>("compare_file");
    if infile == "stdin" && compare_file.map(|s| s == "stdin").unwrap_or(false) {
        anyhow::bail!("only one input file may be \"stdin\" per invocation");
    }
    let trees1 = load_trees(infile)?;

    // 2. Load second file (if provided) or self-compare against trees1
    let trees2_owned: Vec<Tree> = if let Some(f2) = compare_file {
        load_trees(f2)?
    } else {
        Vec::new()
    };
    let trees2: &[Tree] = if compare_file.is_some() {
        &trees2_owned
    } else {
        &trees1
    };

    // Single-file mode requires at least 2 trees for pairwise comparison.
    // Two-file mode can proceed with a single tree on each side.
    // Check before opening the writer so a failed run does not leave an
    // empty output file behind. Empty-file checks fire first so users get a
    // precise "no trees found" message rather than a misleading "need at
    // least 2 trees" when the input is actually empty.
    if trees1.is_empty() {
        anyhow::bail!("no trees found in first input file");
    }
    if compare_file.is_some() && trees2.is_empty() {
        anyhow::bail!("no trees found in second input file");
    }
    if compare_file.is_none() && trees1.len() < 2 {
        anyhow::bail!(
            "need at least 2 trees for pairwise comparison, got {}",
            trees1.len()
        );
    }

    // Pre-validate leaf-set consistency across all trees that will be
    // compared. `compute_tree_metrics` requires exact leaf-set equality; doing
    // this check before opening the writer avoids leaving a partial output
    // file (header + some rows) when a later pair mismatches.
    validate_leaf_consistency(&trees1, "first input file")?;
    if compare_file.is_some() {
        validate_leaf_consistency(trees2, "second input file")?;
        // Cross-file: every tree in file1 must also share the leaf set with
        // every tree in file2. Since each file is internally consistent
        // (checked above), comparing the first tree of each file is enough.
        if !trees1.is_empty() && !trees2.is_empty() {
            let l1 = canonical_leaf_set(&trees1[0])
                .with_context(|| "invalid leaves in tree 1 of first input file")?;
            let l2 = canonical_leaf_set(&trees2[0])
                .with_context(|| "invalid leaves in tree 1 of second input file")?;
            if l1 != l2 {
                let only_l1: Vec<_> = l1.difference(&l2).collect();
                let only_l2: Vec<_> = l2.difference(&l1).collect();
                anyhow::bail!(
                    "leaf sets differ between input files: only in first {:?}, only in second {:?}",
                    only_l1,
                    only_l2
                );
            }
        }
    }

    // 3. Output writer
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    let include_trivial = args.get_flag("include_trivial");

    // 4. Compare
    // Header
    writeln!(writer, "Tree1\tTree2\tRF_Dist\tWRF_Dist\tKF_Dist")?;

    // Single-file mode: skip self-comparisons (j == i) and duplicate pairs
    // (j < i) since RF is symmetric. Two-file mode: full cross comparison.
    for (i, t1) in trees1.iter().enumerate() {
        let start_j = if compare_file.is_some() { 0 } else { i + 1 };
        for (j, t2) in trees2.iter().enumerate().skip(start_j) {
            let (rf, wrf, kf) =
                necom::libs::phylo::cmp::compute_tree_metrics(t1, t2, include_trivial)?;
            writeln!(writer, "{}\t{}\t{}\t{}\t{}", i + 1, j + 1, rf, wrf, kf)?;
        }
    }

    writer.flush()?;
    Ok(())
}
