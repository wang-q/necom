use anyhow::Context;
use clap::{Arg, ArgMatches, Command};
use std::io::Write;

/// All valid comparison methods, in the order used by `--method all`.
const VALID_METHODS: &[&str] =
    &["pearson", "spearman", "mae", "cosine", "jaccard", "euclid"];

/// Build the clap subcommand for compare.
pub fn make_subcommand() -> Command {
    Command::new("compare")
        .about("Compares two distance matrices")
        .after_help(include_str!("../../../docs/help/mat/compare.md"))
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
    let matrix1_file = args
        .get_one::<String>("matrix1")
        .context("missing required argument: matrix1")?;
    let matrix2_file = args
        .get_one::<String>("matrix2")
        .context("missing required argument: matrix2")?;
    let method = args
        .get_one::<String>("mat_method")
        .context("missing required argument: mat_method")?;
    let methods = if method == "all" {
        VALID_METHODS.join(",")
    } else {
        method.clone()
    };

    let mut requested_methods: Vec<&str> = Vec::new();
    for m in methods.split(',').map(str::trim) {
        if m.is_empty() {
            continue;
        }
        if !VALID_METHODS.contains(&m) {
            anyhow::bail!("unknown method: {}", m);
        }
        if !requested_methods.contains(&m) {
            requested_methods.push(m);
        }
    }
    if requested_methods.is_empty() {
        anyhow::bail!("at least one comparison method required");
    }

    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

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
    for method in &requested_methods {
        let score = match *method {
            "pearson" => necom::libs::linalg::pearson_correlation(&values1, &values2),
            "spearman" => necom::libs::linalg::spearman_correlation(&values1, &values2),
            "mae" => necom::libs::linalg::mean_absolute_error(&values1, &values2),
            "cosine" => necom::libs::linalg::cosine_similarity(&values1, &values2),
            "jaccard" => {
                necom::libs::linalg::weighted_jaccard_similarity(&values1, &values2)
            }
            "euclid" => necom::libs::linalg::euclidean_distance(&values1, &values2),
            _ => unreachable!("validated above"),
        };
        // Emit "NA" for non-finite scores (NaN/Inf) to keep TSV output parseable,
        // matching the eval module's format_metrics_row convention.
        if score.is_finite() {
            writeln!(writer, "{}\t{:.6}", method, score)?;
        } else {
            writeln!(writer, "{}\tNA", method)?;
        }
    }

    writer.flush()?;
    Ok(())
}
