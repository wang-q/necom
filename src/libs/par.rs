//! Parallel pipeline primitives shared by `necom mat` subcommands.
//!
//! Provides a writer thread + rayon pool pair, list/path resolution, two-set
//! entry loading, and a generic parallel pairwise iteration helper. None of
//! these depend on clap; the cmd layer extracts positional args and passes
//! them in.

use anyhow::Context;
use rayon::prelude::*;
use rayon::ThreadPool;
use std::io::Write;
use std::thread::JoinHandle;

/// Spawn a writer thread draining a channel and create a local rayon pool with
/// `num_threads`. Returns the sender, the writer join handle, and the pool.
///
/// A local `ThreadPool` is used instead of `build_global` so that the command
/// can be invoked multiple times in the same process (e.g. parallel tests)
/// without failing on global pool re-initialization.
pub fn spawn_writer_and_pool(
    outfile: &str,
    num_threads: usize,
) -> anyhow::Result<(
    crossbeam_channel::Sender<String>,
    JoinHandle<anyhow::Result<()>>,
    ThreadPool,
)> {
    if num_threads == 0 {
        anyhow::bail!("--parallel must be >= 1");
    }

    let (sender, receiver) = crossbeam_channel::bounded::<String>(256);

    let output = outfile.to_string();
    let writer_thread = std::thread::spawn(move || {
        let mut writer = crate::writer(&output)
            .with_context(|| format!("failed to open writer for {}", output))?;
        for result in receiver {
            writer
                .write_all(result.as_bytes())
                .with_context(|| "writer write_all failed")?;
        }
        Ok(())
    });

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .with_context(|| "failed to build rayon thread pool")?;

    Ok((sender, writer_thread, pool))
}

/// Resolve an infile to a list of paths. If `is_list` is true, read the file
/// as a one-path-per-line list; otherwise treat `infile` itself as the path.
pub fn resolve_paths(infile: &str, is_list: bool) -> anyhow::Result<Vec<String>> {
    if is_list {
        crate::libs::io::read_names::<Vec<String>>(infile)
    } else {
        Ok(vec![infile.to_string()])
    }
}

/// Load entries from a list of paths using a per-file loader.
pub fn load_entries<E, F>(paths: &[String], load_fn: F) -> anyhow::Result<Vec<E>>
where
    F: Fn(&str) -> anyhow::Result<Vec<E>>,
{
    let mut entries = Vec::new();
    for path in paths {
        let mut loaded = load_fn(path)?;
        entries.append(&mut loaded);
    }
    Ok(entries)
}

/// Load two entry sets for pairwise comparison.
///
/// With one infile: load it once and return `(entries.clone(), entries)` so
/// the caller can self-compare. With two infiles: load each independently.
/// `load_fn` receives the resolved path list for one set.
pub fn load_two_sets<E, F>(
    infiles: &[&str],
    is_list: bool,
    load_fn: F,
) -> anyhow::Result<(Vec<E>, Vec<E>)>
where
    E: Clone,
    F: Fn(&[String]) -> anyhow::Result<Vec<E>>,
{
    if infiles.len() == 1 {
        let paths = resolve_paths(infiles[0], is_list)?;
        let entries = load_fn(&paths)?;
        Ok((entries.clone(), entries))
    } else {
        let paths1 = resolve_paths(infiles[0], is_list)?;
        let paths2 = resolve_paths(infiles[1], is_list)?;
        let entries1 = load_fn(&paths1)?;
        let entries2 = load_fn(&paths2)?;
        Ok((entries1, entries2))
    }
}

/// Iterate `entries1` x `entries2` in parallel on `pool`, invoking `pair_fn`
/// for each pair. `pair_fn` appends its output directly to the provided
/// `&mut String` buffer (using `write!`/`writeln!`), which is flushed to
/// `sender` every 1000 pairs (and at the end of each row).
///
/// Errors from the writer channel are logged and the worker aborts its row.
pub fn par_run_pairs<E, F>(
    entries1: &[E],
    entries2: &[E],
    sender: &crossbeam_channel::Sender<String>,
    pool: &ThreadPool,
    pair_fn: F,
) where
    E: Sync,
    F: Fn(&E, &E, &mut String) + Sync + Send,
{
    pool.install(|| {
        entries1.par_iter().for_each(|e1| {
            let mut lines = String::with_capacity(1024);
            for (i, e2) in entries2.iter().enumerate() {
                pair_fn(e1, e2, &mut lines);
                if i > 0 && i % 1000 == 0 && !lines.is_empty() {
                    if let Err(e) = sender.send(std::mem::take(&mut lines)) {
                        log::error!("writer channel closed: {}", e);
                        break;
                    }
                }
            }
            if !lines.is_empty() {
                if let Err(e) = sender.send(lines) {
                    log::error!("writer channel closed: {}", e);
                }
            }
        });
    });
}
