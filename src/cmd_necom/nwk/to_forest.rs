use anyhow::Context;
use clap::{ArgMatches, Command};
use necom::libs::phylo::tree::io::to_forest;
use necom::libs::phylo::tree::Tree;
use std::io::Write;

/// Build the clap subcommand for to-forest.
pub fn make_subcommand() -> Command {
    Command::new("to-forest")
        .about("Converts Newick trees to raw LaTeX Forest code")
        .after_help(
            r###"
Convert Newick trees to raw LaTeX Forest code.

This command is designed for manually modifying the generated Forest code.

Notes:
* Styles are stored in the comments of each node
* Drawing a cladogram by default
* Set `--bl` to draw a phylogenetic tree

Examples:
1. Convert to Forest code:
   necom nwk to-forest tests/newick/catarrhini.nwk

2. Convert with branch lengths:
   necom nwk to-forest tests/newick/catarrhini.nwk --bl

3. Save to file:
   necom nwk to-forest tests/newick/catarrhini.nwk -o forest.tex
"###,
        )
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::bl_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the to-forest command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer =
        necom::writer(outfile).with_context(|| format!("Failed to open writer for {}", outfile))?;
    let is_bl = args.get_flag("bl");

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;

    let tree = Tree::from_file(infile)?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("no trees found in {}", infile))?;

    let height = if is_bl {
        tree.get_root()
            .map(|r| tree.get_height(r, true))
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let out_string = to_forest(&tree, height);

    writer.write_all((out_string + "\n").as_ref())?;

    writer.flush()?;
    Ok(())
}
