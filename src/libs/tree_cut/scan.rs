//! Parameter-scan support for tree cutting.

use crate::libs::phylo::tree::Tree;
use crate::libs::tree_cut::{self as cut, CutDispatch};
use std::io::Write;

/// Parameters controlling a `--scan` threshold sweep.
pub struct ScanParams {
    /// Starting threshold value.
    pub start: f64,
    /// Ending threshold value (inclusive).
    pub end: f64,
    /// Step size between threshold values.
    pub step: f64,
    /// Standard method name to sweep, when not using dynamic-tree.
    pub method_name: Option<&'static str>,
    /// Whether the sweep uses dynamic-tree (scan value replaces min cluster size).
    pub dynamic_tree: bool,
}

/// Parse a `--scan` argument of the form `start,end,step`.
pub fn parse_scan_range(scan_str: &str) -> anyhow::Result<(f64, f64, f64)> {
    let parts: Vec<&str> = scan_str.split(',').collect();
    if parts.len() != 3 {
        anyhow::bail!("--scan format must be start,end,step");
    }
    let start: f64 = parts[0].parse()?;
    let end: f64 = parts[1].parse()?;
    let step: f64 = parts[2].parse()?;

    if step <= 0.0 {
        anyhow::bail!("Scan step must be positive");
    }
    Ok((start, end, step))
}

/// Run a parameter sweep over a single tree.
///
/// Writes a long-format table (`Group\tClusterID\tSampleID`) to `writer` and,
/// if a stats writer is provided, summary statistics for each threshold step.
#[allow(clippy::too_many_arguments)]
pub fn run_scan(
    tree: &Tree,
    writer: &mut dyn Write,
    stats_writer: &mut Option<Box<dyn Write>>,
    params: ScanParams,
    deep: usize,
    max_tree_height: Option<f64>,
    deep_split: bool,
    no_pam_dendro: bool,
    max_pam_dist: Option<f64>,
) -> anyhow::Result<()> {
    writer.write_all(b"Group\tClusterID\tSampleID\n")?;

    if let Some(w) = stats_writer.as_deref_mut() {
        w.write_all(b"Group\tClusters\tSingletons\tNon-Singletons\tMaxSize\n")?;
    }

    let n_steps = compute_n_steps(params.start, params.end, params.step)?;
    for i in 0..=n_steps {
        let val = params.start + (i as f64) * params.step;
        if val > params.end + 1e-9 {
            break;
        }

        let dispatch = if params.dynamic_tree {
            build_dynamic_tree_dispatch(
                tree,
                val,
                deep,
                max_tree_height,
                deep_split,
                no_pam_dendro,
                max_pam_dist,
            )?
        } else {
            // Standard method sweep. `method_name` is guaranteed to be Some
            // because the caller validates that a non-dynamic method is present.
            let name = params.method_name.expect("method name required for scan");
            cut::build_dispatch(
                tree,
                Some(name),
                val,
                deep,
                None,
                None,
                max_tree_height,
                deep_split,
                no_pam_dendro,
                max_pam_dist,
                None,
            )?
        };

        let (partition, method_name) = cut::dispatch_cut(tree, dispatch)?;
        let group_label = format!("{}={}", method_name, val);

        if let Some(w) = stats_writer.as_deref_mut() {
            let (n_clusters, n_single, n_non_single, max_size) = partition.get_stats();
            w.write_fmt(format_args!(
                "{}\t{}\t{}\t{}\t{}\n",
                group_label, n_clusters, n_single, n_non_single, max_size
            ))?;
        }

        let rows = cut::format_scan_rows(&partition, tree, &group_label)?;
        writer.write_all(rows.as_bytes())?;
    }

    writer.flush()?;
    Ok(())
}

/// Compute the number of scan steps using integer arithmetic to avoid
/// floating-point drift.
fn compute_n_steps(start: f64, end: f64, step: f64) -> anyhow::Result<i64> {
    let n_steps_f = ((end - start) / step).round();
    if !n_steps_f.is_finite() || n_steps_f < 0.0 || n_steps_f > i64::MAX as f64 {
        anyhow::bail!(
            "scan range too large: start={}, end={}, step={}",
            start,
            end,
            step
        );
    }
    Ok(n_steps_f as i64)
}

/// Build a dispatch for dynamic-tree scan values, validating that the value is
/// a non-negative integer. The scan value replaces the min cluster size at
/// each step (the `--dynamic-tree` argument itself only selects the method).
fn build_dynamic_tree_dispatch(
    tree: &Tree,
    val: f64,
    deep: usize,
    max_tree_height: Option<f64>,
    deep_split: bool,
    no_pam_dendro: bool,
    max_pam_dist: Option<f64>,
) -> anyhow::Result<CutDispatch> {
    if !val.is_finite() || val < 0.0 || val > usize::MAX as f64 {
        anyhow::bail!("scan value out of range: {}", val);
    }
    if val.fract() != 0.0 {
        anyhow::bail!("scan value must be integer for dynamic-tree: {}", val);
    }
    cut::build_dispatch(
        tree,
        None,
        val,
        deep,
        Some(val as usize),
        None,
        max_tree_height,
        deep_split,
        no_pam_dendro,
        max_pam_dist,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scan_range_basic() {
        let (start, end, step) = parse_scan_range("0,0.2,0.1").unwrap();
        assert!((start - 0.0).abs() < 1e-9);
        assert!((end - 0.2).abs() < 1e-9);
        assert!((step - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_parse_scan_range_rejects_non_positive_step() {
        assert!(parse_scan_range("0,1,0").is_err());
        assert!(parse_scan_range("0,1,-0.1").is_err());
    }

    #[test]
    fn test_parse_scan_range_rejects_bad_format() {
        assert!(parse_scan_range("0,1").is_err());
        assert!(parse_scan_range("0,1,2,3").is_err());
    }
}
