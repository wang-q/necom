//! Shared clap argument builders for subcommands.

use clap::{builder, Arg, ArgAction, ArgMatches};

/// Standard `-o/--outfile` argument defaulting to stdout.
pub fn outfile_arg() -> Arg {
    Arg::new("outfile")
        .long("outfile")
        .short('o')
        .num_args(1)
        .default_value("stdout")
        .help("Output filename. [stdout] for screen")
}

/// Required positional `infile` argument (caller must provide, may pass "stdin").
pub fn infile_arg_required() -> Arg {
    infile_arg_required_with_help("Input filename. [stdin] for standard input")
}

/// Required positional `infile` argument with a custom help text.
/// Index is auto-assigned by clap — do not add `.index(N)` to other positionals
/// unless this is the only positional or all positionals use explicit indices.
pub fn infile_arg_required_with_help(help: &'static str) -> Arg {
    Arg::new("infile").required(true).num_args(1).help(help)
}

/// `-i/--invert` flag with a custom help text.
pub fn invert_arg_with_help(help: &'static str) -> Arg {
    Arg::new("invert")
        .long("invert")
        .short('i')
        .action(ArgAction::SetTrue)
        .help(help)
}

/// `--seed` argument (u64) with an optional default, short flag, and help text.
pub fn seed_arg(
    default: Option<&'static str>,
    short: Option<char>,
    help: &'static str,
) -> Arg {
    let arg = Arg::new("seed")
        .long("seed")
        .num_args(1)
        .value_parser(clap::value_parser!(u64))
        .help(help);
    let arg = match default {
        Some(d) => arg.default_value(d),
        None => arg,
    };
    match short {
        Some(c) => arg.short(c),
        None => arg,
    }
}

/// Extract the `outfile` value from `args` as `&str`.
pub fn get_outfile(args: &ArgMatches) -> &str {
    args.get_one::<String>("outfile")
        .map(|s| s.as_str())
        .unwrap_or("stdout")
}

// ============================================================================
// nwk subcommand builders
// ============================================================================

/// Positional `target` tree file argument (required, index 1).
pub fn target_tree_arg(help: &'static str) -> Arg {
    Arg::new("target")
        .required(true)
        .index(1)
        .num_args(1)
        .help(help)
}

/// Standard `--node` (`-n`) selector for nwk subcommands.
pub fn node_arg() -> Arg {
    Arg::new("node")
        .long("node")
        .short('n')
        .num_args(1)
        .action(ArgAction::Append)
        .help("Select nodes by exact name")
}

/// Standard `--name-list` (`-l`) selector for nwk subcommands.
pub fn name_list_arg() -> Arg {
    Arg::new("name_list")
        .long("name-list")
        .short('l')
        .num_args(1)
        .help("Select nodes from a name-list file")
}

/// Standard `--regex` (`-x`) selector for nwk subcommands.
pub fn regex_arg() -> Arg {
    Arg::new("regex")
        .long("regex")
        .short('x')
        .num_args(1)
        .action(ArgAction::Append)
        .help("Select nodes by regular expression (case insensitive)")
}

/// Standard `--descendants` (`-D`) flag for nwk subcommands.
pub fn descendants_arg() -> Arg {
    Arg::new("descendants")
        .long("descendants")
        .short('D')
        .action(ArgAction::SetTrue)
        .help("Include all descendants of selected internal nodes")
}

/// Standard `--internal` (`-I`) filter flag for nwk subcommands.
pub fn internal_arg() -> Arg {
    Arg::new("internal")
        .long("internal")
        .short('I')
        .action(ArgAction::SetTrue)
        .help("Don't print internal labels")
}

/// Standard `--leaf` (`-L`) filter flag for nwk subcommands.
pub fn leaf_arg() -> Arg {
    Arg::new("leaf")
        .long("leaf")
        .short('L')
        .action(ArgAction::SetTrue)
        .help("Don't print leaf labels")
}

/// `-M/--monophyly` flag with a custom help text.
pub fn monophyly_arg(help: &'static str) -> Arg {
    Arg::new("monophyly")
        .long("monophyly")
        .short('M')
        .action(ArgAction::SetTrue)
        .help(help)
}

/// Standard `-b/--bl` flag for nwk subcommands (keep branch lengths in output).
pub fn bl_arg() -> Arg {
    Arg::new("bl")
        .long("bl")
        .short('b')
        .action(ArgAction::SetTrue)
        .help("Keep branch lengths")
}

/// Standard `-l/--lca` argument for nwk subcommands (lowest common ancestor).
pub fn lca_arg() -> Arg {
    Arg::new("lca")
        .long("lca")
        .short('l')
        .num_args(1)
        .action(ArgAction::Append)
        .help("Lowest common ancestor of two nodes")
}

// ============================================================================
// clust subcommand builders
// ============================================================================

/// `--matrix` argument for clust commands (distance matrix file).
pub fn matrix_arg() -> Arg {
    Arg::new("matrix")
        .long("matrix")
        .num_args(1)
        .help("Distance matrix file")
}

/// Standard `--format` argument for clustering output.
pub fn format_arg() -> Arg {
    Arg::new("clust_format")
        .long("format")
        .action(ArgAction::Set)
        .value_parser([
            builder::PossibleValue::new("cluster"),
            builder::PossibleValue::new("pair"),
        ])
        .default_value("cluster")
        .help("Output format for clustering results")
}

/// Standard `--same` argument. `default` varies by algorithm (mcl=1.0, dbscan/k-medoids=0.0).
pub fn same_arg(default: &'static str) -> Arg {
    Arg::new("same")
        .long("same")
        .num_args(1)
        .default_value(default)
        .value_parser(clap::value_parser!(f32))
        .help("Default score of identical element pairs")
}

/// Standard `--missing` argument. `default` varies by algorithm (mcl=0.0, dbscan/k-medoids=1.0).
pub fn missing_arg(default: &'static str) -> Arg {
    Arg::new("missing")
        .long("missing")
        .num_args(1)
        .default_value(default)
        .value_parser(clap::value_parser!(f32))
        .help("Default score of missing pairs")
}

/// `--max-iter` argument (maximum iterations, default 100).
pub fn max_iter_arg() -> Arg {
    Arg::new("max_iter")
        .long("max-iter")
        .num_args(1)
        .default_value("100")
        .value_parser(clap::value_parser!(usize))
        .help("Maximum number of iterations")
}

/// `--method` argument for hierarchical clustering (default: ward).
pub fn clust_method_arg() -> Arg {
    Arg::new("clust_method")
        .long("method")
        .default_value("ward")
        .help("Clustering method (single, complete, average, weighted, centroid, median, ward)")
}

/// `--input-format` argument for clustering partition files (default: pair).
pub fn clust_input_format_arg() -> Arg {
    Arg::new("clust_input_format")
        .long("input-format")
        .value_parser([
            builder::PossibleValue::new("cluster"),
            builder::PossibleValue::new("pair"),
            builder::PossibleValue::new("long"),
        ])
        .default_value("pair")
        .help("Input format for partition files")
}

/// `--eps` argument for DBSCAN (default: 0.05).
pub fn eps_arg() -> Arg {
    Arg::new("eps")
        .long("eps")
        .num_args(1)
        .default_value("0.05")
        .value_parser(clap::value_parser!(f32))
        .help("The maximum distance between two points for DBSCAN clustering")
}

/// `--min-points` argument for DBSCAN (default: 4).
pub fn min_points_arg() -> Arg {
    Arg::new("min_points")
        .long("min-points")
        .num_args(1)
        .default_value("4")
        .value_parser(clap::value_parser!(usize))
        .help("Minimum number of points to form a dense region in DBSCAN")
}

/// `--inflation` argument for MCL (default: 2.0).
pub fn mcl_inflation_arg() -> Arg {
    Arg::new("inflation")
        .long("inflation")
        .short('I')
        .num_args(1)
        .default_value("2.0")
        .value_parser(clap::value_parser!(f64))
        .help("Inflation parameter. Controls the granularity of clusters. Higher values = tighter/more clusters.")
}

/// `--prune` argument for MCL (default: 1e-5).
pub fn mcl_prune_arg() -> Arg {
    Arg::new("prune")
        .long("prune")
        .num_args(1)
        .default_value("1e-5")
        .value_parser(clap::value_parser!(f64))
        .help("Pruning threshold. Matrix entries smaller than this will be set to zero.")
}

/// `--runs` argument for randomized clustering (default: 10).
pub fn runs_arg() -> Arg {
    Arg::new("runs")
        .long("runs")
        .num_args(1)
        .default_value("10")
        .value_parser(clap::value_parser!(usize))
        .help("Number of random initializations")
}

/// `--rep` argument for tree cut representative selection (default: root).
pub fn rep_arg() -> Arg {
    Arg::new("rep")
        .long("rep")
        .num_args(1)
        .value_parser([
            builder::PossibleValue::new("root"),
            builder::PossibleValue::new("first"),
            builder::PossibleValue::new("medoid"),
        ])
        .default_value("root")
        .help("Representative selection method")
}

/// `--rep` argument for flat clustering representative selection (default: medoid).
pub fn flat_rep_arg() -> Arg {
    Arg::new("flat_rep")
        .long("rep")
        .num_args(1)
        .value_parser([
            builder::PossibleValue::new("medoid"),
            builder::PossibleValue::new("first"),
        ])
        .default_value("medoid")
        .help("Representative selection method")
}

/// `--scan` argument for parameter sweep (format: start,end,step).
pub fn scan_arg() -> Arg {
    Arg::new("scan")
        .long("scan")
        .num_args(1)
        .help("Scan thresholds (format: start,end,step)")
}

/// `--stats-out` argument for scan summary output.
pub fn stats_out_arg() -> Arg {
    Arg::new("stats_out")
        .long("stats-out")
        .num_args(1)
        .help("Output statistics to a separate file (useful when format is 'long')")
}

/// `--support` argument for branch support threshold.
pub fn support_arg() -> Arg {
    Arg::new("support")
        .long("support")
        .num_args(1)
        .value_parser(clap::value_parser!(f64))
        .help(
            "Branch support threshold (edges with support < S will be treated as infinite length). \
             Internal node names that cannot be parsed as numbers are treated as support = 100.0.",
        )
}

/// `--deep` argument for inconsistent coefficient depth (default: 2).
pub fn deep_arg() -> Arg {
    Arg::new("deep")
        .long("deep")
        .num_args(1)
        .default_value("2")
        .value_parser(clap::value_parser!(usize))
        .help("Depth for inconsistent coefficient calculation (default: 2)")
}

/// `--dynamic-tree` argument for dynamic tree cut (value: min cluster size).
pub fn dynamic_tree_arg() -> Arg {
    Arg::new("dynamic_tree")
        .long("dynamic-tree")
        .num_args(1)
        .value_parser(clap::value_parser!(usize))
        .help("Use dynamic tree cut method (value: min cluster size)")
}

/// `--dynamic-hybrid` argument for dynamic hybrid cut (value: min cluster size).
pub fn dynamic_hybrid_arg() -> Arg {
    Arg::new("dynamic_hybrid")
        .long("dynamic-hybrid")
        .num_args(1)
        .value_parser(clap::value_parser!(usize))
        .help("Use dynamic hybrid cut method (value: min cluster size)")
}

/// `--max-pam-dist` argument for hybrid cut PAM reassignment.
pub fn max_pam_dist_arg() -> Arg {
    Arg::new("max_pam_dist")
        .long("max-pam-dist")
        .num_args(1)
        .value_parser(clap::value_parser!(f64))
        .help("Maximum distance to medoid for PAM reassignment")
}

/// `--no-pam-dendro` flag for hybrid cut.
pub fn no_pam_dendro_arg() -> Arg {
    Arg::new("no_pam_dendro")
        .long("no-pam-dendro")
        .action(ArgAction::SetTrue)
        .help("Disable dendrogram respect in PAM stage (allow assigning to clusters across high branches)")
}

/// `--deep-split` flag for dynamic tree cut.
pub fn deep_split_arg() -> Arg {
    Arg::new("deep_split")
        .long("deep-split")
        .action(ArgAction::SetTrue)
        .help("Enable deep split for dynamic tree cut (default: false)")
}

/// `--max-tree-height` argument for dynamic tree cut.
pub fn max_tree_height_arg() -> Arg {
    Arg::new("max_tree_height")
        .long("max-tree-height")
        .num_args(1)
        .value_parser(clap::value_parser!(f64))
        .help(
            "Maximum joining height for dynamic tree cut (default: 99% of tree height)",
        )
}

/// `--other` argument for external partition evaluation (alias: `--truth`).
pub fn other_partition_arg() -> Arg {
    Arg::new("other")
        .long("other")
        .alias("truth")
        .num_args(1)
        .help("Other partition file (for external evaluation)")
}

/// `--tree` argument for internal evaluation using patristic distance.
pub fn tree_arg() -> Arg {
    Arg::new("tree").long("tree").num_args(1).help(
        "Tree file (for internal evaluation: Silhouette, using patristic distance)",
    )
}

/// `--coords` argument for internal evaluation using Davies-Bouldin.
pub fn coords_arg() -> Arg {
    Arg::new("coords")
        .long("coords")
        .num_args(1)
        .help("Coordinate matrix file (for internal evaluation: Davies-Bouldin)")
}

/// `--no-singletons` flag for external evaluation.
pub fn no_singletons_arg() -> Arg {
    Arg::new("no_singletons")
        .long("no-singletons")
        .action(ArgAction::SetTrue)
        .help("Exclude true singletons (from Reference/Ground Truth) from evaluation")
}

// ============================================================================
// mat subcommand builders
// ============================================================================

/// `--method` argument for matrix comparison (default: pearson).
/// Accepts comma-separated methods (e.g. "pearson,cosine") or "all".
/// Validation is done by the caller (each token checked against known methods).
pub fn mat_method_arg() -> Arg {
    Arg::new("mat_method")
        .long("method")
        .action(ArgAction::Set)
        .default_value("pearson")
        .help("Comparison method(s), comma-separated (all|pearson|spearman|mae|cosine|jaccard|euclid)")
}

/// `--format` argument for matrix output (default: full).
pub fn mat_format_arg() -> Arg {
    Arg::new("mat_format")
        .long("format")
        .action(ArgAction::Set)
        .value_parser([
            builder::PossibleValue::new("full"),
            builder::PossibleValue::new("lower"),
            builder::PossibleValue::new("strict"),
        ])
        .default_value("full")
        .help("Output format")
}

/// `--input-format` argument for matrix transform (default: phylip).
pub fn mat_input_format_arg() -> Arg {
    Arg::new("mat_input_format")
        .long("input-format")
        .default_value("phylip")
        .value_parser([
            builder::PossibleValue::new("phylip"),
            builder::PossibleValue::new("pair"),
        ])
        .help("Input format")
}

// ============================================================================
// Additional common builders
// ============================================================================

/// `--replace-tsv` argument (required) for replace commands.
pub fn replace_tsv_arg() -> Arg {
    Arg::new("replace_tsv")
        .long("replace-tsv")
        .required(true)
        .num_args(1)
        .help("TSV file of original_name and replacement_name(s)")
}

/// `--mode` argument with possible values, a default, and a custom help text.
pub fn mode_arg(
    default: &'static str,
    possible: &'static [&'static str],
    help: &'static str,
) -> Arg {
    let values: Vec<builder::PossibleValue> = possible
        .iter()
        .map(|v| builder::PossibleValue::new(*v))
        .collect();
    Arg::new("mode")
        .long("mode")
        .num_args(1)
        .action(ArgAction::Set)
        .default_value(default)
        .value_parser(values)
        .help(help)
}

// ============================================================================
// mat subcommand additional builders
// ============================================================================

/// Positional `name_list` file argument (used by `mat subset`).
pub fn mat_name_list_arg(required: bool) -> Arg {
    Arg::new("name_list")
        .required(required)
        .index(2)
        .num_args(1)
        .help(if required {
            "File containing one sequence name per line"
        } else {
            "File containing one sequence name per line (optional)"
        })
}

// ============================================================================
// clust subcommand additional builders
// ============================================================================

/// `-k/--k` number of clusters argument.
pub fn k_arg() -> Arg {
    Arg::new("k")
        .long("k")
        .short('k')
        .num_args(1)
        .value_parser(clap::value_parser!(usize))
        .help("Number of clusters")
}

// ============================================================================
// Cross-domain shared builders
// ============================================================================

/// `--color` argument (no short flag, optional default value).
pub fn color_arg(default: Option<&'static str>, help: &'static str) -> Arg {
    let arg = Arg::new("color").long("color").num_args(1);
    match default {
        Some(d) => arg.default_value(d),
        None => arg,
    }
    .help(help)
}
