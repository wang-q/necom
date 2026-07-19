use anyhow::{Context, Result};
use clap::{ArgMatches, Command};
use necom::libs::phylo::tree::Tree;
use necom::libs::tree_cut::{self as cut, RepMode, METHOD_NAMES};
use std::io::Write;

pub mod dynamic;
pub mod hybrid;
pub mod scan_dynamic;
pub mod scan_simple;
pub mod simple;

/// Build the clap subcommand for cut.
pub fn make_subcommand() -> Command {
    Command::new("cut")
        .about("Cuts a tree into flat partitions")
        .after_help(include_str!("../../../docs/help/cut.md"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(simple::make_subcommand())
        .subcommand(dynamic::make_subcommand())
        .subcommand(hybrid::make_subcommand())
        .subcommand(scan_simple::make_subcommand())
        .subcommand(scan_dynamic::make_subcommand())
}

/// Execute the cut command.
pub fn execute(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        Some(("simple", sub_matches)) => simple::execute(sub_matches),
        Some(("dynamic", sub_matches)) => dynamic::execute(sub_matches),
        Some(("hybrid", sub_matches)) => hybrid::execute(sub_matches),
        Some(("scan-simple", sub_matches)) => scan_simple::execute(sub_matches),
        Some(("scan-dynamic", sub_matches)) => scan_dynamic::execute(sub_matches),
        _ => anyhow::bail!("unrecognized cut subcommand"),
    }
}

/// Load the input tree and apply optional support filtering.
pub fn load_tree(args: &ArgMatches) -> Result<Tree> {
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("missing required argument: infile"))?;

    let mut trees = Tree::from_file(infile)?;
    if trees.len() > 1 {
        anyhow::bail!(
            "Input file contains multiple trees. Only single tree input is supported."
        );
    }
    if trees.is_empty() {
        anyhow::bail!("Input file contains no tree");
    }

    if let Some(&support_threshold) = args.get_one::<f64>("support") {
        cut::apply_support_filter(&mut trees[0], support_threshold);
    }

    Ok(trees.into_iter().next().expect("single tree"))
}

/// Common output options for simple/dynamic/hybrid cuts.
pub struct OutputOptions<'a> {
    /// Output filename (`stdout` for screen).
    pub outfile: &'a str,
    /// Output format (`cluster` or `pair`).
    pub format: &'a str,
    /// Representative selection mode.
    pub rep_mode: RepMode,
}

impl<'a> OutputOptions<'a> {
    /// Extract output options from parsed arguments.
    pub fn from_args(args: &'a ArgMatches) -> Result<Self> {
        let outfile = crate::cmd_necom::args::get_outfile(args);
        let format = args.get_one::<String>("clust_format").unwrap();
        let rep_method = args.get_one::<String>("rep").unwrap().as_str();
        let rep_mode = RepMode::parse(rep_method)?;
        Ok(Self {
            outfile,
            format,
            rep_mode,
        })
    }
}

/// Write formatted clusters to the configured output destination.
pub fn write_clusters(clusters: &[cut::Cluster], opts: &OutputOptions) -> Result<()> {
    let output = cut::format_clusters(clusters, opts.format)?;
    let mut writer = necom::writer(opts.outfile)
        .with_context(|| format!("Failed to open writer for {}", opts.outfile))?;
    writer.write_all(output.as_bytes())?;
    Ok(())
}

/// Open the `--stats-out` writer for scan commands.
pub fn init_stats_writer(args: &ArgMatches) -> Result<Option<Box<dyn Write>>> {
    if let Some(stats_file) = args.get_one::<String>("stats_out") {
        let w = Box::new(
            necom::writer(stats_file)
                .with_context(|| format!("Failed to open writer for {}", stats_file))?,
        );
        Ok(Some(w))
    } else {
        Ok(None)
    }
}

/// Normalize a user-provided method name to the internal underscore form and
/// validate it against `METHOD_NAMES`.
pub fn normalize_method_name(name: &str) -> Result<&'static str> {
    let normalized = name.replace('-', "_");
    METHOD_NAMES
        .iter()
        .find(|&&n| n == normalized)
        .copied()
        .ok_or_else(|| anyhow::anyhow!("unknown method: {}", name))
}

/// Command-layer builder wrapper around `tree_cut::build_dispatch`.
///
/// Converts named parameters to the library's positional call to avoid
/// confusion with the many `None`/`false` defaults.
pub struct DispatchBuilder {
    method_name: Option<&'static str>,
    val: f64,
    deep: usize,
    dynamic_tree: Option<usize>,
    dynamic_hybrid: Option<usize>,
    max_tree_height: Option<f64>,
    deep_split: bool,
    no_pam_dendro: bool,
    max_pam_dist: Option<f64>,
    matrix: Option<necom::libs::pairmat::NamedMatrix>,
}

impl DispatchBuilder {
    /// Builder for standard (simple) methods.
    pub fn standard(method_name: &'static str, val: f64, deep: usize) -> Self {
        Self {
            method_name: Some(method_name),
            val,
            deep,
            dynamic_tree: None,
            dynamic_hybrid: None,
            max_tree_height: None,
            deep_split: false,
            no_pam_dendro: false,
            max_pam_dist: None,
            matrix: None,
        }
    }

    /// Builder for dynamic tree cut.
    pub fn dynamic(min_size: usize) -> Self {
        Self {
            method_name: None,
            val: 0.0,
            deep: 2,
            dynamic_tree: Some(min_size),
            dynamic_hybrid: None,
            max_tree_height: None,
            deep_split: false,
            no_pam_dendro: false,
            max_pam_dist: None,
            matrix: None,
        }
    }

    /// Builder for hybrid cut.
    pub fn hybrid(min_size: usize, matrix: necom::libs::pairmat::NamedMatrix) -> Self {
        Self {
            method_name: None,
            val: 0.0,
            deep: 2,
            dynamic_tree: None,
            dynamic_hybrid: Some(min_size),
            max_tree_height: None,
            deep_split: false,
            no_pam_dendro: false,
            max_pam_dist: None,
            matrix: Some(matrix),
        }
    }

    /// Set the maximum tree height.
    pub fn max_tree_height(mut self, height: Option<f64>) -> Self {
        self.max_tree_height = height;
        self
    }

    /// Enable deep split.
    pub fn deep_split(mut self, enable: bool) -> Self {
        self.deep_split = enable;
        self
    }

    /// Disable dendrogram respect in PAM stage.
    pub fn no_pam_dendro(mut self, disable: bool) -> Self {
        self.no_pam_dendro = disable;
        self
    }

    /// Set maximum PAM reassignment distance.
    pub fn max_pam_dist(mut self, dist: Option<f64>) -> Self {
        self.max_pam_dist = dist;
        self
    }

    /// Build the `CutDispatch`.
    pub fn build(self, tree: &Tree) -> Result<cut::CutDispatch> {
        cut::build_dispatch(
            tree,
            self.method_name,
            self.val,
            self.deep,
            self.dynamic_tree,
            self.dynamic_hybrid,
            self.max_tree_height,
            self.deep_split,
            self.no_pam_dendro,
            self.max_pam_dist,
            self.matrix,
        )
    }
}
