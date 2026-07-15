use anyhow::Context;
use clap::{builder::PossibleValue, value_parser, Arg, ArgAction, ArgMatches, Command};
use std::io::Write;
/// Build the clap subcommand for transform.
pub fn make_subcommand() -> Command {
    Command::new("transform")
        .about("Applies mathematical transformations to a matrix")
        .after_help(
            r###"
Transform matrix values element-wise.
Useful for converting similarity matrices to distance matrices.

Operations:
    * linear:     val = val * scale + offset
    * inv-linear: val = max - val
    * log:        val = -ln(val)
    * exp:        val = exp(-val)
    * square:     val = val * val
    * sqrt:       val = sqrt(val)

Normalization:
    If --normalize is set, values are normalized using diagonal elements before transformation:
    x_norm(i, j) = x(i, j) / sqrt(x(i, i) * x(j, j))

Examples:
    1. Convert Identity (0-100) to Distance (0-1):
       # Using linear: -0.01 * x + 1.0 = (100 - x) / 100
       necom mat transform in.phy --op linear --scale -0.01 --offset 1.0

    2. Convert Identity (0-100) to Distance (0-100):
       necom mat transform in.phy --op inv-linear --max-val 100

    3. Convert Similarity (0-1) to Distance (0-1):
       necom mat transform in.phy --op inv-linear --max-val 1.0

    4. Log transformation with normalization (e.g. for probability):
       necom mat transform in.phy --op log --normalize
"###,
        )
        .arg(crate::cmd_necom::args::infile_arg_required_with_help(
            "Input PHYLIP matrix or pairwise TSV file",
        ))
        .arg(
            Arg::new("op")
                .long("op")
                .default_value("linear")
                .value_parser([
                    PossibleValue::new("linear"),
                    PossibleValue::new("inv-linear"),
                    PossibleValue::new("log"),
                    PossibleValue::new("exp"),
                    PossibleValue::new("square"),
                    PossibleValue::new("sqrt"),
                ])
                .help("Transformation operation"),
        )
        .arg(
            Arg::new("max_val")
                .long("max-val")
                .default_value("1.0")
                .value_parser(value_parser!(f32))
                .help("Maximum value for inv-linear"),
        )
        .arg(
            Arg::new("scale")
                .long("scale")
                .default_value("1.0")
                .value_parser(value_parser!(f32))
                .help("Scale factor for linear"),
        )
        .arg(
            Arg::new("offset")
                .long("offset")
                .default_value("0.0")
                .value_parser(value_parser!(f32))
                .help("Offset value for linear"),
        )
        .arg(
            Arg::new("normalize")
                .long("normalize")
                .action(ArgAction::SetTrue)
                .help("Normalize based on diagonal values"),
        )
        .arg(crate::cmd_necom::args::mat_input_format_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}
/// Execute the transform command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args.get_one::<String>("infile").unwrap();
    let op = args.get_one::<String>("op").unwrap().as_str();
    let max_val = *args.get_one::<f32>("max_val").unwrap();
    let scale = *args.get_one::<f32>("scale").unwrap();
    let offset = *args.get_one::<f32>("offset").unwrap();
    let normalize = args.get_flag("normalize");
    let format = args.get_one::<String>("mat_input_format").unwrap().as_str();
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer =
        necom::writer(outfile).with_context(|| format!("Failed to open writer for {}", outfile))?;

    // Load and Transform
    let matrix = if format == "pair" {
        necom::libs::pairmat::NamedMatrix::from_pair_scores(infile, 0.0, 1.0)?
    } else {
        necom::libs::pairmat::NamedMatrix::from_relaxed_phylip(infile)?
    };

    let matrix =
        necom::libs::pairmat::transform_matrix(&matrix, op, max_val, scale, offset, normalize)?;

    let size = matrix.size();
    let names = matrix.get_names();

    writer.write_fmt(format_args!("{:>4}\n", size))?;
    for (i, name) in names.iter().enumerate() {
        writer.write_fmt(format_args!("{}", name))?;
        for j in 0..size {
            let val = matrix.get(i, j);
            writer.write_fmt(format_args!("\t{:.6}", val))?;
        }
        writer.write_fmt(format_args!("\n"))?;
    }

    writer.flush()?;
    Ok(())
}
