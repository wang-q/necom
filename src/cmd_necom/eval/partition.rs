use anyhow::Context;
use clap::{Arg, ArgMatches, Command};
use necom::libs::eval::{
    load_batch_partitions, load_partition, remove_singletons, run_batch, run_single,
    Coordinates, DistanceMatrix, EvalTarget, PartitionFormat, TreeDistance,
};
use necom::libs::pairmat::NamedMatrix;
use necom::libs::phylo::tree::Tree;
use std::io::Write;
/// Build the clap subcommand for partition.
pub fn make_subcommand() -> Command {
    Command::new("partition")
        .about("Evaluates clustering quality")
        .after_help(include_str!("../../../docs/help/eval/partition.md"))
        .arg(
            Arg::new("p1")
                .required(true)
                .index(1)
                .help("Partition file"),
        )
        .arg(crate::cmd_necom::args::other_partition_arg())
        .arg(crate::cmd_necom::args::matrix_arg())
        .arg(crate::cmd_necom::args::tree_arg())
        .arg(crate::cmd_necom::args::coords_arg())
        .arg(crate::cmd_necom::args::clust_input_format_arg())
        .arg(crate::cmd_necom::args::other_format_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
        .arg(crate::cmd_necom::args::no_singletons_arg())
}
/// Execute the partition command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let p1_path = args
        .get_one::<String>("p1")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: p1"))?;
    let outfile = crate::cmd_necom::args::get_outfile(args);

    // clust_input_format has a clap default value, so unwrap is safe.
    let format_str = args.get_one::<String>("clust_input_format").unwrap();
    let format: PartitionFormat = match format_str.parse() {
        Ok(f) => f,
        Err(e) => anyhow::bail!("Invalid format: {}", e),
    };

    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;

    let remove_singletons_flag = args.get_flag("no_singletons");

    // Enforce mutual exclusion of evaluation targets. This applies uniformly
    // to both single and batch modes, so that users get an explicit error
    // instead of silently dropped targets (single mode previously used an
    // `else if` chain with priority `other > matrix > tree > coords`, while
    // batch mode collected all targets — the two modes disagreed).
    let provided: [&str; 4] = ["other", "matrix", "tree", "coords"];
    let count = provided
        .iter()
        .filter(|k| args.get_one::<String>(k).is_some())
        .count();
    if count > 1 {
        anyhow::bail!(
            "only one of --other/--truth, --matrix, --tree, --coords may be provided; got {}",
            count
        );
    }

    if format == PartitionFormat::Long {
        // Batch Mode
        let batches = load_batch_partitions(p1_path)?;

        // Determine the format for `--other`. In batch mode p1 is Long
        // (multi-partition), so the truth file cannot also be Long.
        // `--other-format` overrides; otherwise default to `Cluster`.
        let other_format = match args.get_one::<String>("other_format") {
            Some(s) => match s.parse::<PartitionFormat>() {
                Ok(f) => f,
                Err(e) => anyhow::bail!("Invalid --other-format: {}", e),
            },
            None => PartitionFormat::Cluster,
        };

        // Prepare resources (I/O stays in cmd layer).
        let p2 = if let Some(p2_path) = args.get_one::<String>("other") {
            let mut truth = load_partition(p2_path, other_format)?;
            if remove_singletons_flag {
                remove_singletons(&mut truth);
            }
            Some(truth)
        } else {
            None
        };

        let dist_provider: Option<Box<dyn DistanceMatrix>> =
            if let Some(matrix_path) = args.get_one::<String>("matrix") {
                Some(Box::new(NamedMatrix::from_relaxed_phylip(matrix_path)?))
            } else if let Some(tree_path) = args.get_one::<String>("tree") {
                let trees = Tree::from_file(tree_path)?;
                if trees.len() != 1 {
                    anyhow::bail!("Tree file must contain exactly one tree.");
                }
                // trees.len() == 1 verified above; .next() is guaranteed Some.
                let tree = trees.into_iter().next().unwrap();
                Some(Box::new(TreeDistance::new(tree)))
            } else {
                None
            };

        let coords = if let Some(coords_path) = args.get_one::<String>("coords") {
            Some(Coordinates::from_path(coords_path)?)
        } else {
            None
        };

        if p2.is_none() && dist_provider.is_none() && coords.is_none() {
            anyhow::bail!(
                "Batch mode requires at least one evaluation target: --other/--truth, --matrix, --tree, or --coords."
            );
        }

        let mut targets: Vec<EvalTarget<'_>> = vec![];
        if let Some(ref truth) = p2 {
            targets.push(EvalTarget::External(truth));
        }
        if let Some(ref d) = dist_provider {
            targets.push(EvalTarget::Matrix(&**d));
        }
        if let Some(ref c) = coords {
            targets.push(EvalTarget::Coords(c));
        }

        run_batch(batches, &targets, &mut writer)?;
        writer.flush()?;
        return Ok(());
    }

    // Single Mode
    let p1 = load_partition(p1_path, format)?;

    // `--other-format` overrides the format for the `--other` file in single
    // mode too. Defaults to the same format as p1 when not specified.
    let other_format = match args.get_one::<String>("other_format") {
        Some(s) => match s.parse::<PartitionFormat>() {
            Ok(f) => f,
            Err(e) => anyhow::bail!("Invalid --other-format: {}", e),
        },
        None => format,
    };

    // Mutual exclusion is already enforced above, so at most one branch fires.
    if let Some(p2_path) = args.get_one::<String>("other") {
        let mut p2 = load_partition(p2_path, other_format)?;
        if remove_singletons_flag {
            remove_singletons(&mut p2);
        }
        run_single(&p1, EvalTarget::External(&p2), &mut writer)?;
    } else if let Some(matrix_path) = args.get_one::<String>("matrix") {
        let matrix = NamedMatrix::from_relaxed_phylip(matrix_path)?;
        run_single(&p1, EvalTarget::Matrix(&matrix), &mut writer)?;
    } else if let Some(tree_path) = args.get_one::<String>("tree") {
        let trees = Tree::from_file(tree_path)?;
        if trees.len() != 1 {
            anyhow::bail!("Tree file must contain exactly one tree.");
        }
        // trees.len() == 1 verified above; .next() is guaranteed Some.
        let tree = trees.into_iter().next().unwrap();
        let dist = TreeDistance::new(tree);
        run_single(&p1, EvalTarget::Matrix(&dist), &mut writer)?;
    } else if let Some(coords_path) = args.get_one::<String>("coords") {
        let coords = Coordinates::from_path(coords_path)?;
        run_single(&p1, EvalTarget::Coords(&coords), &mut writer)?;
    } else {
        anyhow::bail!(
            "Either --other/--truth (for external eval), --matrix, --tree, or --coords (for internal eval) must be provided."
        );
    }

    writer.flush()?;
    Ok(())
}
