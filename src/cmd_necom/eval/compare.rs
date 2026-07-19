use anyhow::Context;
use clap::{Arg, ArgAction, ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
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
                .num_args(1)
                .index(2)
                .help("Second input filename (optional)"),
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

/// Execute the compare command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // 1. Load first file
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let trees1 = load_trees(infile)?;

    // 2. Load second file (if provided) or self-compare against trees1
    let compare_file = args.get_one::<String>("compare_file");
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
    // empty output file behind.
    if compare_file.is_none() && trees1.len() < 2 {
        anyhow::bail!(
            "need at least 2 trees for pairwise comparison, got {}",
            trees1.len()
        );
    }
    if trees1.is_empty() {
        anyhow::bail!("no trees found in first input file");
    }
    if compare_file.is_some() && trees2.is_empty() {
        anyhow::bail!("no trees found in second input file");
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
