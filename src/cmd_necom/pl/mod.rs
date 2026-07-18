pub mod condense;

use clap::{ArgMatches, Command};

/// Build the clap subcommand for pl.
pub fn make_subcommand() -> Command {
    Command::new("pl")
        .about("Runs integrated pipelines")
        .after_help(include_str!("../../../docs/help/pl.md"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(condense::make_subcommand())
}

/// Execute the pl command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    match args.subcommand() {
        Some(("condense", sub_matches)) => condense::execute(sub_matches),
        _ => anyhow::bail!("unrecognized pl subcommand"),
    }
}
