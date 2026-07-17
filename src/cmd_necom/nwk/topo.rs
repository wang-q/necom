use anyhow::Context;
use clap::{Arg, ArgAction, ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
use std::io::Write;

/// Build the clap subcommand for topo.
pub fn make_subcommand() -> Command {
    Command::new("topo")
        .about("Manipulates tree topology and attributes")
        .after_help(include_str!("../../../docs/help/nwk/topo.md"))
        .arg(crate::cmd_necom::args::infile_arg_required())
        .arg(crate::cmd_necom::args::bl_arg())
        .arg(
            Arg::new("comment")
                .long("comment")
                .short('c')
                .action(ArgAction::SetTrue)
                .help("Keep comments"),
        )
        .arg(crate::cmd_necom::args::internal_arg())
        .arg(crate::cmd_necom::args::leaf_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the topo command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    let is_bl = args.get_flag("bl");
    let is_comment = args.get_flag("comment");
    let skip_internal = args.get_flag("internal");
    let skip_leaf = args.get_flag("leaf");

    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;
    let trees = Tree::from_file(infile)?;

    for mut tree in trees {
        tree.strip_topology(is_bl, is_comment, skip_internal, skip_leaf);

        let out_string = tree.to_newick();
        writer.write_fmt(format_args!("{}\n", out_string))?;
    }

    writer.flush()?;
    Ok(())
}
