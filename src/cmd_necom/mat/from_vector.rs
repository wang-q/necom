use clap::{Arg, ArgAction, ArgMatches, Command};
use std::sync::{Arc, Mutex};

use necom::libs::linalg;

/// Build the clap subcommand for vector.
pub fn make_subcommand() -> Command {
    Command::new("from-vector")
        .about("Calculates similarity/distance between vectors")
        .after_help(include_str!("../../../docs/help/mat/from-vector.md"))
        .arg(crate::cmd_necom::args::pair_infiles_arg())
        .arg(crate::cmd_necom::args::mode_arg(
            "euclid",
            &["euclid", "cosine", "jaccard"],
            "Mode of calculation",
        ))
        .arg(
            Arg::new("binary")
                .long("binary")
                .action(ArgAction::SetTrue)
                .help("Treat values in list as binary (0 or 1)"),
        )
        .arg(crate::cmd_necom::args::sim_arg())
        .arg(
            Arg::new("dis")
                .long("dis")
                .action(ArgAction::SetTrue)
                .help("Convert to dissimilarity"),
        )
        .arg(crate::cmd_necom::args::parallel_arg())
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the vector command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let opt_mode = args.get_one::<String>("mode").unwrap();

    let is_bin = args.get_flag("binary");
    let is_sim = args.get_flag("sim");
    let is_dis = args.get_flag("dis");

    let opt_parallel = *args.get_one::<usize>("parallel").unwrap();

    let infiles = crate::cmd_necom::args::collect_infiles(args);

    let (sender, writer_thread) = necom::libs::par::spawn_writer_and_pool(
        crate::cmd_necom::args::get_outfile(args),
        opt_parallel,
    )?;

    let (entries1, entries2) =
        necom::libs::par::load_two_sets(&infiles, false, |paths| {
            necom::libs::clust::feature::load_feature_vectors(&paths[0], is_bin)
        })?;

    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let errors_clone = errors.clone();

    necom::libs::par::par_run_pairs(&entries1, &entries2, &sender, |e1, e2| {
        match linalg::vector_score(e1.list(), e2.list(), opt_mode, is_sim, is_dis) {
            Ok(score) => {
                let line = format!("{}\t{}\t{:.4}\n", e1.name(), e2.name(), score);
                Some(line)
            }
            Err(e) => {
                let msg = format!("{} vs {}: {}", e1.name(), e2.name(), e);
                log::error!("{}", msg);
                let mut guard = errors_clone
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guard.push(msg);
                None
            }
        }
    });

    // Drop the sender to signal the writer thread to exit
    drop(sender);
    // Wait for the writer thread to finish
    writer_thread.join().map_err(|e| {
        let msg = e
            .downcast_ref::<String>()
            .map(|s| s.as_str())
            .or_else(|| e.downcast_ref::<&str>().copied())
            .unwrap_or("<non-string panic payload>");
        anyhow::anyhow!("writer thread panicked: {}", msg)
    })?;

    let errors = errors
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !errors.is_empty() {
        anyhow::bail!(
            "vector scoring failed for {} pair(s): {}",
            errors.len(),
            errors[0]
        );
    }

    Ok(())
}
