use anyhow::Context;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use cmd_lib::{run_cmd, run_fun};
use necom::libs::phylo::tree::Tree;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::{Read, Write};

/// Build the clap subcommand for condense.
pub fn make_subcommand() -> Command {
    Command::new("condense")
        .about("Condenses subtrees based on taxonomy")
        .after_help(include_str!("../../../docs/help/pl/condense.md"))
        .arg(
            crate::cmd_necom::args::infile_arg_required_with_help(
                "Input Newick filename. [stdin] for standard input",
            ),
        )
        .arg(
            Arg::new("taxon")
                .long("taxon")
                .num_args(1)
                .required(true)
                .help("Path to taxonomy TSV file"),
        )
        .arg(
            Arg::new("rank")
                .long("rank")
                .num_args(1)
                .action(ArgAction::Append)
                .value_parser(value_parser!(usize))
                .help("Column index(es) to use for grouping (1-based, can be specified multiple times, default: 2)"),
        )
        .arg(
            Arg::new("map")
                .long("map")
                .action(ArgAction::SetTrue)
                .help("Write a map file `condensed.tsv`"),
        )
        .arg(crate::cmd_necom::args::outfile_arg())
}

/// Execute the condense command.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = crate::cmd_necom::args::get_outfile(args);
    let taxon_file = args.get_one::<String>("taxon").unwrap();

    let ranks: Vec<usize> = if args.contains_id("rank") {
        args.get_many::<usize>("rank").unwrap().copied().collect()
    } else {
        vec![2] // default to column 2
    };
    for rank_col in &ranks {
        anyhow::ensure!(*rank_col >= 1, "--rank must be >= 1, got {}", rank_col);
    }

    let ctx = necom::libs::pl::PipelineCtx::new("necom_condense_")?;
    let exe = ctx.necom.clone();

    // Operating
    run_cmd!(info "==> Absolute paths")?;
    let infile = args.get_one::<String>("infile").unwrap();
    // Resolve the input path before entering tempdir (CWD changes on enter).
    // For stdin, defer handling until after entering tempdir so the content can
    // be buffered to a file and read multiple times by subprocesses.
    let abs_infile_opt = if infile == "stdin" {
        None
    } else {
        Some(ctx.abs_path(infile)?)
    };
    let abs_taxon = ctx.abs_path(taxon_file)?;

    let _cwd_guard = ctx.enter()?;

    // For stdin: read once into tempdir/input.nwk so both Tree::from_file and
    // the `nwk indent` subprocess can read it. stdin cannot be rewound.
    let abs_infile = match abs_infile_opt {
        Some(p) => p,
        None => {
            let input_path = ctx.tempdir.path().join("input.nwk");
            let mut reader = necom::reader("stdin")?;
            let mut content = String::new();
            reader
                .read_to_string(&mut content)
                .with_context(|| "Failed to read from stdin")?;
            fs::write(&input_path, &content)
                .with_context(|| "Failed to write stdin content to input.nwk")?;
            necom::libs::io::path_to_str(&input_path)?.to_string()
        }
    };

    // Read tree leaf names for filtering
    let trees = Tree::from_file(&abs_infile)?;
    let leaf_names: BTreeSet<String> = if let Some(tree) = trees.first() {
        tree.get_leaf_names().into_iter().flatten().collect()
    } else {
        BTreeSet::new()
    };
    let leaf_count = leaf_names.len();
    run_cmd!(info "    Tree leaf count: ${leaf_count}")?;

    // Read taxonomy TSV
    run_cmd!(info "==> Read taxonomy TSV")?;

    let taxon_reader = necom::reader(&abs_taxon)
        .with_context(|| format!("Failed to open reader for {}", abs_taxon))?;
    let taxon_table =
        necom::libs::phylo::read_taxonomy(taxon_reader, &ranks, &leaf_names)?;
    let taxon_map = taxon_table.taxon_map;
    let all_groups = taxon_table.all_groups;

    let taxon_count = taxon_map.len();
    run_cmd!(info "    Loaded ${taxon_count} taxonomy entries")?;
    for (i, rank) in ranks.iter().enumerate() {
        let rank_groups = all_groups[i].len();
        run_cmd!(info "    Rank ${rank}: ${rank_groups} groups")?;
    }

    // Start - copy input tree
    run_cmd!(info "==> Start")?;
    run_cmd!(
        ${exe} nwk indent ${abs_infile} -o start.nwk
    )?;

    // Condensing - process each rank
    run_cmd!(info "==> Condensing")?;
    let mut cur_tree = "start.nwk".to_string();
    let mut condensed: Vec<String> = vec![];
    let mut toggle = false;

    // Pre-build reverse index: (rank_idx, group) -> Vec<node_name>
    // Avoids O(n^2) re-filtering of taxon_map for every (rank, group) pair.
    let mut group_nodes: HashMap<(usize, String), Vec<String>> = HashMap::new();
    for (name, terms) in taxon_map.iter() {
        for (rank_idx, term) in terms.iter().enumerate() {
            if let Some(t) = term.as_deref() {
                group_nodes
                    .entry((rank_idx, t.to_string()))
                    .or_default()
                    .push(name.clone());
            }
        }
    }

    for (rank_idx, groups) in all_groups.iter().enumerate() {
        let rank_num = ranks[rank_idx];
        run_cmd!(info "    Processing rank ${rank_num}")?;

        for group in groups.iter() {
            // Find all original nodes that belong to this group at this rank
            let nodes_in_group: Vec<String> = group_nodes
                .get(&(rank_idx, group.clone()))
                .cloned()
                .unwrap_or_default();

            if nodes_in_group.len() < 2 {
                continue;
            }

            // Write node list to a reusable file
            let mut writer = necom::writer("nodes.txt")
                .with_context(|| "Failed to open writer for nodes.txt")?;
            for node in &nodes_in_group {
                writeln!(writer, "{}", node)?;
            }
            writer.flush()?;
            drop(writer);

            // Check if these nodes form a monophyletic group and get labels.
            // nwk label -M exits 0 with empty output for non-monophyletic groups;
            // a non-zero exit indicates a real error and is propagated via `?`.
            let labels_output = run_fun!(
                ${exe} nwk label ${cur_tree} -l nodes.txt -M
            )?;

            let labels: Vec<String> = labels_output
                .split('\n')
                .map(|s: &str| s.to_string())
                .filter(|s: &String| !s.is_empty())
                .collect();

            if labels.is_empty() {
                // Not monophyletic, skip
                continue;
            }

            let new_label = format!("{}||{}", group, nodes_in_group.len());

            // Record mapping: original node name -> condensed label
            for node in &nodes_in_group {
                condensed.push(format!("{}\t{}", node, new_label));
            }

            // Condense the subtree; use alternating output files to avoid clobbering
            toggle = !toggle;
            let new_tree = if toggle {
                "temp_a.nwk".to_string()
            } else {
                "temp_b.nwk".to_string()
            };
            run_cmd!(
                ${exe} nwk subtree ${cur_tree} -l nodes.txt -M --condense ${new_label} -o ${new_tree}
            )?;

            cur_tree = new_tree;
        }
    }

    // Results
    run_cmd!(info "==> Results")?;

    let mut writer = necom::writer("condensed.tsv")
        .with_context(|| "Failed to open writer for condensed.tsv")?;
    for line in condensed.iter() {
        writeln!(writer, "{}", line)?;
    }
    writer.flush()?;

    // Restore original CWD before writing outputs to relative paths (e.g.
    // --map's condensed.tsv, or a relative -o outfile). Intermediate files
    // (temp trees, condensed.tsv in tempdir) are accessed via absolute paths
    // from here on.
    drop(_cwd_guard);

    // Done: copy directly from the final temp tree (cur_tree) to the outfile.
    let final_tree_path = ctx.tempdir.path().join(&cur_tree);
    if outfile == "stdout" {
        let result_content = fs::read_to_string(&final_tree_path)
            .with_context(|| "Failed to read from final tree")?;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        write!(out, "{}", result_content)?;
        out.flush()?;
    } else {
        fs::copy(necom::libs::io::path_to_str(&final_tree_path)?, outfile)?;
    }

    if args.get_flag("map") {
        fs::copy(
            necom::libs::io::path_to_str(&ctx.tempdir.path().join("condensed.tsv"))?,
            "condensed.tsv",
        )?;
    }

    Ok(())
}
