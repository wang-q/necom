use anyhow::Context;
use clap::{Arg, ArgMatches, Command};
use std::io::Write;
/// Build the clap subcommand for compare.
pub fn make_subcommand() -> Command {
    Command::new("compare")
        .about("Compares two distance matrices")
        .after_help(
            r###"
Compare two PHYLIP distance matrices and calculate similarity metrics.

Methods:
* all:       Calculate all metrics below
* pearson:   Pearson correlation coefficient (-1 to 1)
* spearman:  Spearman rank correlation (-1 to 1)
* mae:       Mean absolute error
* cosine:    Cosine similarity (-1 to 1)
* jaccard:   Weighted Jaccard similarity (0 to 1)
* euclid:    Euclidean distance

Examples:
1. Compare using Pearson correlation:
   necom mat compare matrix1.phy matrix2.phy --method pearson

2. Compare using multiple methods:
   necom mat compare matrix1.phy matrix2.phy --method pearson,cosine,jaccard
"###,
        )
        .arg(
            Arg::new("matrix1")
                .required(true)
                .index(1)
                .help("First PHYLIP matrix file"),
        )
        .arg(
            Arg::new("matrix2")
                .required(true)
                .index(2)
                .help("Second PHYLIP matrix file"),
        )
        .arg(crate::cmd_necom::args::mat_method_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Execute the compare command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let matrix1_file = args.get_one::<String>("matrix1").unwrap();
    let matrix2_file = args.get_one::<String>("matrix2").unwrap();
    let method = args.get_one::<String>("mat_method").unwrap();
    let methods = if method == "all" {
        "pearson,spearman,mae,cosine,jaccard,euclid"
    } else {
        method.as_str()
    };
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer =
        necom::writer(outfile).with_context(|| format!("Failed to open writer for {}", outfile))?;

    // Load matrices
    let matrix1 = necom::libs::pairmat::NamedMatrix::from_relaxed_phylip(matrix1_file)?;
    let matrix2 = necom::libs::pairmat::NamedMatrix::from_relaxed_phylip(matrix2_file)?;

    // Report sequence counts
    log::info!(
        "Sequences in matrices: {} and {}",
        matrix1.size(),
        matrix2.size()
    );

    // Extract paired values from common lower triangle
    let (common_names, values1, values2) =
        necom::libs::pairmat::extract_common_lower_triangle(&matrix1, &matrix2)?;

    log::info!("Common sequences: {}", common_names.len());

    // Write header
    writer.write_all(b"Method\tScore\n")?;

    // Calculate and output metrics
    for method in methods.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let result = match method {
            "pearson" => necom::libs::linalg::pearson_correlation(&values1, &values2),
            "spearman" => necom::libs::linalg::spearman_correlation(&values1, &values2),
            "mae" => necom::libs::linalg::mean_absolute_error(&values1, &values2),
            "cosine" => necom::libs::linalg::cosine_similarity(&values1, &values2),
            "jaccard" => necom::libs::linalg::weighted_jaccard_similarity(&values1, &values2),
            "euclid" => necom::libs::linalg::euclidean_distance(&values1, &values2),
            _ => anyhow::bail!("unknown method: {}", method),
        };
        writer.write_fmt(format_args!("{}\t{:.6}\n", method, result))?;
    }

    writer.flush()?;
    Ok(())
}
