use anyhow::Context;
use clap::{builder::PossibleValue, value_parser, Arg, ArgAction, ArgMatches, Command};
/// Build the clap subcommand for transform.
pub fn make_subcommand() -> Command {
    Command::new("transform")
        .about("Applies mathematical transformations to a matrix")
        .after_help(include_str!("../../../docs/help/mat/transform.md"))
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
        .arg(crate::cmd_necom::args::same_arg("0.0"))
        .arg(crate::cmd_necom::args::missing_arg("1.0"))
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
    let opt_same = *args.get_one::<f32>("same").unwrap();
    let opt_missing = *args.get_one::<f32>("missing").unwrap();
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    // Load and Transform
    let matrix = if format == "pair" {
        necom::libs::pairmat::NamedMatrix::from_pair_scores(
            infile,
            opt_same,
            opt_missing,
        )?
    } else {
        necom::libs::pairmat::NamedMatrix::from_relaxed_phylip(infile)?
    };

    let matrix = necom::libs::pairmat::transform_matrix(
        &matrix, op, max_val, scale, offset, normalize,
    )?;

    necom::libs::pairmat::write_phylip_matrix(
        &matrix,
        necom::libs::pairmat::MatrixFormat::Full,
        Some(6),
        &mut writer,
    )?;

    writer.flush()?;
    Ok(())
}
