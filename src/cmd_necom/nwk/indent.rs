use anyhow::Context;
use clap::{Arg, ArgAction, ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
use std::io::Write;

/// Build the clap subcommand for indent.
pub fn make_subcommand() -> Command {
    Command::new("indent")
        .about("Formats Newick trees with indentation")
        .after_help(include_str!("../../../docs/help/nwk/indent.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(
            Arg::new("text")
                .long("text")
                .num_args(1)
                .default_value("  ")
                .help("Use this text instead of the default two spaces"),
        )
        .arg(crate::cmd_necom::args::outfile_arg())
        .arg(
            Arg::new("compact")
                .long("compact")
                .short('c')
                .action(ArgAction::SetTrue)
                .help("Compact output (remove indentation)"),
        )
}

/// Execute the indent command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    let compact = args.get_flag("compact");
    let text = if compact {
        ""
    } else {
        args.get_one::<String>("text")
            .ok_or_else(|| anyhow::anyhow!("missing required argument: text"))?
    };

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let trees = Tree::from_file(infile)?;

    for tree in trees {
        let out_string = tree.to_newick_with_format(text);
        writer.write_all((out_string + "\n").as_ref())?;
    }

    writer.flush()?;
    Ok(())
}
