use clap::{ArgMatches, Command};

pub mod compare;
pub mod partition;
pub mod replicate;

/// Build the clap subcommand for eval.
pub fn make_subcommand() -> Command {
    Command::new("eval")
        .about("Evaluates clustering partitions and phylogenetic trees")
        .after_help(include_str!("../../../docs/help/eval.md"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(compare::make_subcommand())
        .subcommand(partition::make_subcommand())
        .subcommand(replicate::make_subcommand())
}
/// Execute the eval command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    match args.subcommand() {
        Some(("compare", sub_matches)) => compare::execute(sub_matches),
        Some(("partition", sub_matches)) => partition::execute(sub_matches),
        Some(("replicate", sub_matches)) => replicate::execute(sub_matches),
        _ => anyhow::bail!("unrecognized eval subcommand"),
    }
}
