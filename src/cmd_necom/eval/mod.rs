use clap::{ArgMatches, Command};

pub mod compare;

/// Build the clap subcommand for eval.
pub fn make_subcommand() -> Command {
    Command::new("eval")
        .about("Evaluates clustering partitions and phylogenetic trees")
        .after_help(
            r###"
This suite of tools provides evaluation metrics for clustering partitions
and phylogenetic trees.

Subcommand groups:
* Tree comparison: compare

Notes:
* Reads from stdin if input file is 'stdin'

"###,
        )
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(compare::make_subcommand())
}
/// Execute the eval command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    match args.subcommand() {
        Some(("compare", sub_matches)) => compare::execute(sub_matches),
        _ => anyhow::bail!("unrecognized eval subcommand"),
    }
}
