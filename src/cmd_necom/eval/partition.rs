use anyhow::Context;
use clap::{Arg, ArgMatches, Command};
use necom::libs::eval::{
    load_batch_partitions, load_partition, remove_singletons, run_batch, run_single,
    Coordinates, DistanceMatrix, EvalTarget, LabelMap, PartitionFormat, TreeDistance,
};
use necom::libs::pairmat::NamedMatrix;
use necom::libs::phylo::tree::Tree;
use std::collections::HashSet;
use std::io::Write;

/// Verify that every item in `partition` has a coordinate vector.
///
/// Prevents silent dropping of samples in coordinate-based metrics; if any
/// partition item is missing from the coordinate file, the command bails with
/// a clear message instead of producing misleading scores.
fn ensure_coords_cover_partition(
    partition: &LabelMap,
    coords: &Coordinates,
) -> anyhow::Result<()> {
    let missing: Vec<&str> = partition
        .keys()
        .filter(|k| !coords.contains(k))
        .map(|k| k.as_str())
        .collect();
    if !missing.is_empty() {
        anyhow::bail!(
            "{} sample(s) from the partition are missing in --coords (showing up to 5): {:?}",
            missing.len(),
            missing.iter().take(5).copied().collect::<Vec<_>>()
        );
    }
    Ok(())
}

/// Verify that every item in `partition` is present in `names`.
///
/// Prevents silent `NaN` metrics when the distance source (matrix or tree)
/// does not contain a partition sample.
fn ensure_names_cover_partition(
    partition: &LabelMap,
    names: &HashSet<String>,
    source: &str,
) -> anyhow::Result<()> {
    let missing: Vec<&str> = partition
        .keys()
        .filter(|k| !names.contains(*k))
        .map(|k| k.as_str())
        .collect();
    if !missing.is_empty() {
        anyhow::bail!(
            "{} sample(s) from the partition are missing in {} (showing up to 5): {:?}",
            missing.len(),
            source,
            missing.iter().take(5).copied().collect::<Vec<_>>()
        );
    }
    Ok(())
}

/// Verify that `partition` is not empty.
fn ensure_non_empty(partition: &LabelMap, context: &str) -> anyhow::Result<()> {
    if partition.is_empty() {
        anyhow::bail!("{} is empty (no samples found)", context);
    }
    Ok(())
}

/// Verify that two partitions cover exactly the same sample set.
///
/// External metrics require the two partitions to be comparable; silently
/// intersecting keys can hide mismatched inputs and produce misleading scores.
/// `p1_context` is used in error messages (e.g. "p1" or a batch group id).
fn ensure_partitions_align(
    p1: &LabelMap,
    p2: &LabelMap,
    p1_context: &str,
) -> anyhow::Result<()> {
    let only_in_p1: Vec<&str> = p1
        .keys()
        .filter(|k| !p2.contains_key(*k))
        .map(|k| k.as_str())
        .collect();
    let only_in_p2: Vec<&str> = p2
        .keys()
        .filter(|k| !p1.contains_key(*k))
        .map(|k| k.as_str())
        .collect();
    if !only_in_p1.is_empty() || !only_in_p2.is_empty() {
        anyhow::bail!(
            "partition sample sets do not match: {} only in {} {:?}, {} only in the other partition {:?}",
            only_in_p1.len(),
            p1_context,
            only_in_p1.iter().take(5).copied().collect::<Vec<_>>(),
            only_in_p2.len(),
            only_in_p2.iter().take(5).copied().collect::<Vec<_>>()
        );
    }
    Ok(())
}

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

    let remove_singletons_flag = args.get_flag("no_singletons");

    // `--no-singletons` only affects the reference partition (`--other`).
    // Reject it in other contexts so users are not misled into thinking it
    // filters the input partition or distance-source samples.
    if remove_singletons_flag && args.get_one::<String>("other").is_none() {
        anyhow::bail!("--no-singletons requires --other/--truth");
    }

    // Only one input stream may be stdin at a time; otherwise the second read
    // sees EOF and produces a confusing "empty file" error.
    let stdin_count = [
        Some(p1_path.as_str()),
        args.get_one::<String>("other").map(|s| s.as_str()),
        args.get_one::<String>("matrix").map(|s| s.as_str()),
        args.get_one::<String>("tree").map(|s| s.as_str()),
        args.get_one::<String>("coords").map(|s| s.as_str()),
    ]
    .iter()
    .filter(|&&p| p == Some("stdin"))
    .count();
    if stdin_count > 1 {
        anyhow::bail!("only one input file may be \"stdin\" per invocation");
    }

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

    // The output writer is opened only after all input loading and validation
    // has succeeded (see `run_batch` / `run_single` call sites below). Opening
    // it earlier would truncate an existing outfile when a later step fails.

    if format == PartitionFormat::Long {
        // Batch Mode
        let batches = load_batch_partitions(p1_path)?;
        if batches.is_empty() {
            anyhow::bail!("no batch groups found in partition file");
        }

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
            // Catches both "empty --other file" and "--no-singletons removed
            // every cluster" cases. Without this, every batch group would be
            // retained down to an empty set, alignment would succeed on two
            // empty sets, and `run_batch` would emit misleading all-zero rows.
            ensure_non_empty(&truth, "partition --other")?;
            Some(truth)
        } else {
            None
        };

        let dist_provider: Option<Box<dyn DistanceMatrix>> =
            if let Some(matrix_path) = args.get_one::<String>("matrix") {
                let matrix = NamedMatrix::from_relaxed_phylip(matrix_path)?;
                let matrix_names: HashSet<String> =
                    matrix.get_names().into_iter().cloned().collect();
                for (_, p1) in &batches {
                    ensure_names_cover_partition(p1, &matrix_names, "--matrix")?;
                }
                Some(Box::new(matrix))
            } else if let Some(tree_path) = args.get_one::<String>("tree") {
                let trees = Tree::from_file(tree_path)?;
                if trees.len() != 1 {
                    anyhow::bail!("Tree file must contain exactly one tree.");
                }
                // trees.len() == 1 verified above; .next() is guaranteed Some.
                let tree = trees.into_iter().next().unwrap();
                let tree_names: HashSet<String> =
                    tree.get_leaf_names().into_iter().flatten().collect();
                for (_, p1) in &batches {
                    ensure_names_cover_partition(p1, &tree_names, "--tree")?;
                }
                Some(Box::new(TreeDistance::new(tree)?))
            } else {
                None
            };

        let coords = if let Some(coords_path) = args.get_one::<String>("coords") {
            Some(Coordinates::from_path(coords_path)?)
        } else {
            None
        };

        if let Some(ref c) = coords {
            for (_, p1) in &batches {
                ensure_coords_cover_partition(p1, c)?;
            }
        }

        if p2.is_none() && dist_provider.is_none() && coords.is_none() {
            anyhow::bail!(
                "Batch mode requires at least one evaluation target: --other/--truth, --matrix, --tree, or --coords."
            );
        }

        // Validate groups and align external partitions before running.
        let mut processed_batches: Vec<(String, LabelMap)> =
            Vec::with_capacity(batches.len());
        for (group, mut p1) in batches {
            ensure_non_empty(&p1, &format!("group {}", group))?;
            if let Some(ref truth) = p2 {
                if remove_singletons_flag {
                    // Samples removed from the reference by --no-singletons
                    // are excluded from evaluation rather than treated as a
                    // mismatch.
                    p1.retain(|k, _| truth.contains_key(k));
                    // After filtering, the group may have become empty. Catch
                    // this explicitly so the user gets a clear message instead
                    // of a generic "sample sets do not match" error.
                    ensure_non_empty(
                        &p1,
                        &format!("group {} after --no-singletons filtering", group),
                    )?;
                }
                ensure_partitions_align(&p1, truth, &group)?;
            }
            processed_batches.push((group, p1));
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

        // Open the writer only after all inputs have been loaded and validated.
        let mut writer = necom::writer(outfile)
            .with_context(|| format!("Failed to open writer for {}", outfile))?;
        run_batch(processed_batches, &targets, &mut writer)?;
        writer.flush()?;
        return Ok(());
    }

    // Single Mode
    let p1 = load_partition(p1_path, format)?;
    ensure_non_empty(&p1, "partition p1")?;

    // `--other-format` overrides the format for the `--other` file in single
    // mode too. Defaults to the same format as p1 when not specified.
    let other_format = match args.get_one::<String>("other_format") {
        Some(s) => match s.parse::<PartitionFormat>() {
            Ok(f) => f,
            Err(e) => anyhow::bail!("Invalid --other-format: {}", e),
        },
        None => format,
    };

    // Compute the evaluation target (and adjust p1 for `--other` mode) before
    // opening the writer, so that input loading/validation failures do not
    // truncate an existing outfile. `Prepared` owns both the (possibly
    // filtered) p1 and the target; `SingleTarget::as_eval_target` borrows the
    // target so `run_single` can consume the prepared pair after the writer is
    // opened.
    enum SingleTarget {
        External(LabelMap),
        Matrix(NamedMatrix),
        Tree(TreeDistance),
        Coords(Coordinates),
    }

    impl SingleTarget {
        fn as_eval_target(&self) -> EvalTarget<'_> {
            match self {
                SingleTarget::External(p) => EvalTarget::External(p),
                SingleTarget::Matrix(m) => EvalTarget::Matrix(m),
                SingleTarget::Tree(t) => EvalTarget::Matrix(t),
                SingleTarget::Coords(c) => EvalTarget::Coords(c),
            }
        }
    }

    struct Prepared {
        p1: LabelMap,
        target: SingleTarget,
    }

    // Mutual exclusion is already enforced above, so at most one branch fires.
    let prepared = if let Some(p2_path) = args.get_one::<String>("other") {
        let mut p2 = load_partition(p2_path, other_format)?;
        if remove_singletons_flag {
            remove_singletons(&mut p2);
        }
        // Catches both "empty --other file" and "--no-singletons removed every
        // cluster" cases. Without this, p1_for_eval and p2 could both end up
        // empty, `ensure_partitions_align` would succeed on two empty sets,
        // and `run_single` would emit misleading all-zero metrics.
        ensure_non_empty(&p2, "partition --other")?;
        // Filter p1 to match p2 after singleton removal; otherwise require an
        // exact sample-set match so users are not misled by silent intersection.
        let mut p1_for_eval = p1.clone();
        if remove_singletons_flag {
            p1_for_eval.retain(|k, _| p2.contains_key(k));
        }
        ensure_partitions_align(&p1_for_eval, &p2, "p1")?;
        Prepared {
            p1: p1_for_eval,
            target: SingleTarget::External(p2),
        }
    } else if let Some(matrix_path) = args.get_one::<String>("matrix") {
        let matrix = NamedMatrix::from_relaxed_phylip(matrix_path)?;
        let matrix_names: HashSet<String> =
            matrix.get_names().into_iter().cloned().collect();
        ensure_names_cover_partition(&p1, &matrix_names, "--matrix")?;
        Prepared {
            p1,
            target: SingleTarget::Matrix(matrix),
        }
    } else if let Some(tree_path) = args.get_one::<String>("tree") {
        let trees = Tree::from_file(tree_path)?;
        if trees.len() != 1 {
            anyhow::bail!("Tree file must contain exactly one tree.");
        }
        // trees.len() == 1 verified above; .next() is guaranteed Some.
        let tree = trees.into_iter().next().unwrap();
        let tree_names: HashSet<String> =
            tree.get_leaf_names().into_iter().flatten().collect();
        ensure_names_cover_partition(&p1, &tree_names, "--tree")?;
        let dist = TreeDistance::new(tree)?;
        Prepared {
            p1,
            target: SingleTarget::Tree(dist),
        }
    } else if let Some(coords_path) = args.get_one::<String>("coords") {
        let coords = Coordinates::from_path(coords_path)?;
        ensure_coords_cover_partition(&p1, &coords)?;
        Prepared {
            p1,
            target: SingleTarget::Coords(coords),
        }
    } else {
        anyhow::bail!(
            "Either --other/--truth (for external eval), --matrix, --tree, or --coords (for internal eval) must be provided."
        );
    };

    // Open the writer only after all inputs have been loaded and validated.
    let mut writer = necom::writer(outfile)
        .with_context(|| format!("Failed to open writer for {}", outfile))?;
    run_single(&prepared.p1, prepared.target.as_eval_target(), &mut writer)?;
    writer.flush()?;
    Ok(())
}
