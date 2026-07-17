use anyhow::Context;
use clap::{Arg, ArgAction, ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
use std::io::Write;

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
/// Execute the compare command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // 1. Load first file
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let trees1 = Tree::from_file(infile)?;

    // 2. Load second file (if provided) or self-compare against trees1
    let compare_file = args.get_one::<String>("compare_file");
    let trees2_owned: Vec<Tree> = if let Some(f2) = compare_file {
        Tree::from_file(f2)?
    } else {
        Vec::new()
    };
    let trees2: &[Tree] = if compare_file.is_some() {
        &trees2_owned
    } else {
        &trees1
    };

    // 3. Output writer
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    let include_trivial = args.get_flag("include_trivial");

    // Warn if single-file mode has fewer than 2 trees
    if compare_file.is_none() && trees1.len() < 2 {
        log::warn!(
            "need at least 2 trees for pairwise comparison, got {}",
            trees1.len()
        );
    }

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
