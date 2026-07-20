//! Parameter-scan support for tree cutting.

use crate::libs::cut::{self as cut, CutDispatch};
use crate::libs::phylo::tree::Tree;
use std::io::Write;

/// Parameters controlling a scan threshold sweep.
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

/// Parse a `--range` argument of the form `start,end,step` for floating-point methods.
pub fn parse_scan_range(scan_str: &str) -> anyhow::Result<(f64, f64, f64)> {
    let parts: Vec<&str> = scan_str.split(',').collect();
    if parts.len() != 3 {
        anyhow::bail!("--range format must be start,end,step");
    }
    let start: f64 = parts[0].parse()?;
    let end: f64 = parts[1].parse()?;
    let step: f64 = parts[2].parse()?;

    if !start.is_finite() || !end.is_finite() || !step.is_finite() {
        anyhow::bail!("--range values must be finite numbers");
    }
    if start > end {
        anyhow::bail!("--range start must not exceed end");
    }
    if step <= 0.0 {
        anyhow::bail!("--range step must be positive");
    }
    Ok((start, end, step))
}

/// Parse a `--range` argument of the form `start,end,step` for integer methods.
pub fn parse_scan_range_usize(scan_str: &str) -> anyhow::Result<(usize, usize, usize)> {
    let parts: Vec<&str> = scan_str.split(',').collect();
    if parts.len() != 3 {
        anyhow::bail!("--range format must be start,end,step");
    }

    let labels = ["start", "end", "step"];
    let mut values = [0usize; 3];
    for (i, part) in parts.iter().enumerate() {
        match part.parse::<usize>() {
            Ok(v) => values[i] = v,
            Err(_) => {
                anyhow::bail!(
                    "--range {} must be a non-negative integer, got {}",
                    labels[i],
                    part
                );
            }
        }
    }

    if values[0] > values[1] {
        anyhow::bail!("--range start must not exceed end");
    }
    if values[2] == 0 {
        anyhow::bail!("--range step must be positive");
    }

    Ok((values[0], values[1], values[2]))
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
    // Validate the scan range before writing any output headers. Previously
    // the headers were written first, which left truncated header-only output
    // files on disk when `compute_n_steps` rejected an oversized range.
    let n_steps = compute_n_steps(params.start, params.end, params.step)?;

    writer.write_all(b"Group\tClusterID\tSampleID\n")?;

    if let Some(w) = stats_writer.as_deref_mut() {
        w.write_all(b"Group\tClusters\tSingletons\tNon-Singletons\tMaxSize\n")?;
    }

    let mut values: Vec<f64> = Vec::with_capacity((n_steps + 2) as usize);
    for i in 0..=n_steps {
        values.push(params.start + (i as f64) * params.step);
    }
    // Ensure the explicit end value is included even when step does not
    // evenly divide the interval (e.g. 0,10,3 should emit 10).
    if let Some(&last) = values.last() {
        if params.end > last + 1e-9 {
            values.push(params.end);
        }
    }

    for val in values {
        let dispatch = if params.dynamic_tree {
            build_dynamic_tree_dispatch(tree, val, max_tree_height, deep_split)?
        } else {
            // Standard method sweep. `method_name` is guaranteed to be Some
            // because the caller validates that a non-dynamic method is present.
            let name = params.method_name.ok_or_else(|| {
                anyhow::anyhow!("method name required for non-dynamic scan")
            })?;
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
        // Format `val` with fixed precision then strip trailing zeros so that
        // floating-point drift (e.g. 0.1+0.1+0.1 = 0.30000000000000004) does
        // not leak into the Group label.
        let group_label = format!("{}={}", method_name, format_scan_value(val));

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
    let n_steps_f = ((end - start) / step).floor();
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

/// Format a scan threshold value for display, stripping trailing zeros and
/// the decimal point so that floating-point drift (e.g. `0.1 + 0.1 + 0.1`
/// yielding `0.30000000000000004`) does not leak into Group labels.
fn format_scan_value(val: f64) -> String {
    let s = format!("{:.10}", val);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Build a dispatch for dynamic-tree scan values, validating that the value is
/// a non-negative integer. The scan value replaces the min cluster size at
/// each step (the `--dynamic-tree` argument itself only selects the method).
fn build_dynamic_tree_dispatch(
    tree: &Tree,
    val: f64,
    max_tree_height: Option<f64>,
    deep_split: bool,
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
        2,
        Some(val as usize),
        None,
        max_tree_height,
        deep_split,
        false,
        None,
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

    #[test]
    fn test_parse_scan_range_rejects_start_greater_than_end() {
        let err = parse_scan_range("0.5,0.2,0.1").unwrap_err().to_string();
        assert!(err.contains("start must not exceed end"), "got: {}", err);
    }

    #[test]
    fn test_parse_scan_range_rejects_non_finite_values() {
        assert!(parse_scan_range("inf,1,0.1").is_err());
        assert!(parse_scan_range("0,inf,0.1").is_err());
        assert!(parse_scan_range("0,1,inf").is_err());
        assert!(parse_scan_range("nan,1,0.1").is_err());
    }

    #[test]
    fn test_parse_scan_range_usize_rejects_start_greater_than_end() {
        let err = parse_scan_range_usize("5,2,1").unwrap_err().to_string();
        assert!(err.contains("start must not exceed end"), "got: {}", err);
    }

    #[test]
    fn test_format_scan_value_strips_drift() {
        // Floating-point drift from 0.1+0.1+0.1 must not appear in the label.
        assert_eq!(format_scan_value(0.30000000000000004), "0.3");
        assert_eq!(format_scan_value(0.6000000000000001), "0.6");
        assert_eq!(format_scan_value(0.7000000000000001), "0.7");
    }

    #[test]
    fn test_format_scan_value_integers_and_clean_values() {
        assert_eq!(format_scan_value(0.0), "0");
        assert_eq!(format_scan_value(1.0), "1");
        assert_eq!(format_scan_value(3.0), "3");
        assert_eq!(format_scan_value(0.1), "0.1");
        assert_eq!(format_scan_value(0.25), "0.25");
    }

    #[test]
    fn test_run_scan_includes_end_when_step_not_divisor() {
        // Tree: ((A:0.1,B:0.1):0.1,C:0.2);
        let tree =
            crate::libs::phylo::tree::Tree::from_newick("((A:0.1,B:0.1):0.1,C:0.2);")
                .expect("valid newick");
        let mut output = Vec::new();
        let mut stats: Option<Box<dyn std::io::Write>> = None;
        let params = ScanParams {
            start: 0.0,
            end: 0.2,
            step: 0.15,
            method_name: Some("height"),
            dynamic_tree: false,
        };
        run_scan(
            &tree,
            &mut output,
            &mut stats,
            params,
            2,
            None,
            false,
            false,
            None,
        )
        .expect("scan should succeed");
        let out = String::from_utf8(output).expect("valid utf-8");
        let groups: Vec<&str> = out
            .lines()
            .skip(1)
            .filter_map(|l| l.split('\t').next())
            .collect();
        // 0.15 does not divide 0.2; the explicit end 0.2 must still appear.
        assert!(groups.contains(&"height=0"), "missing height=0");
        assert!(groups.contains(&"height=0.15"), "missing height=0.15");
        assert!(groups.contains(&"height=0.2"), "missing height=0.2");
    }

    #[test]
    fn test_run_scan_no_duplicate_end_when_step_divides_evenly() {
        // Tree: ((A:0.1,B:0.1):0.1,C:0.2);
        let tree =
            crate::libs::phylo::tree::Tree::from_newick("((A:0.1,B:0.1):0.1,C:0.2);")
                .expect("valid newick");
        let mut output = Vec::new();
        let mut stats: Option<Box<dyn std::io::Write>> = None;
        let params = ScanParams {
            start: 0.0,
            end: 0.2,
            step: 0.1,
            method_name: Some("height"),
            dynamic_tree: false,
        };
        run_scan(
            &tree,
            &mut output,
            &mut stats,
            params,
            2,
            None,
            false,
            false,
            None,
        )
        .expect("scan should succeed");
        let out = String::from_utf8(output).expect("valid utf-8");
        let groups: std::collections::HashSet<&str> = out
            .lines()
            .skip(1)
            .filter_map(|l| l.split('\t').next())
            .collect();
        assert_eq!(
            groups.iter().filter(|&&g| g == "height=0.2").count(),
            1,
            "end value must appear exactly once"
        );
        assert!(groups.contains("height=0"));
        assert!(groups.contains("height=0.1"));
        assert!(groups.contains("height=0.2"));
    }

    /// Regression: `run_scan` must return an error (not panic) when
    /// `dynamic_tree == false` and `method_name == None`. Previously this path
    /// called `.expect()` which would crash the process.
    #[test]
    fn test_run_scan_missing_method_name_returns_error() {
        let tree =
            crate::libs::phylo::tree::Tree::from_newick("((A:0.1,B:0.1):0.1,C:0.2);")
                .expect("valid newick");
        let mut output = Vec::new();
        let mut stats: Option<Box<dyn std::io::Write>> = None;
        let params = ScanParams {
            start: 0.0,
            end: 0.1,
            step: 0.1,
            method_name: None,
            dynamic_tree: false,
        };
        let result = run_scan(
            &tree,
            &mut output,
            &mut stats,
            params,
            2,
            None,
            false,
            false,
            None,
        );
        assert!(result.is_err(), "expected error, got {:?}", result);
        let msg = result.err().unwrap().to_string();
        assert!(
            msg.contains("method name required"),
            "unexpected error message: {}",
            msg
        );
    }

    /// Regression: `run_scan` must validate the scan range *before* writing
    /// the output header. Previously, an oversized range (rejected by
    /// `compute_n_steps`) would still leave a header-only file on disk for
    /// the caller's writer. The fix moves `compute_n_steps` ahead of any
    /// `write_all` calls so that on validation failure no bytes are written.
    #[test]
    fn test_run_scan_oversized_range_writes_no_header() {
        let tree =
            crate::libs::phylo::tree::Tree::from_newick("((A:0.1,B:0.1):0.1,C:0.2);")
                .expect("valid newick");
        let mut output = Vec::new();
        let mut stats: Option<Box<dyn std::io::Write>> = None;
        // Range too large: (1e308 - 0.0) / 1e-10 > i64::MAX.
        let params = ScanParams {
            start: 0.0,
            end: 1e308,
            step: 1e-10,
            method_name: Some("height"),
            dynamic_tree: false,
        };
        let result = run_scan(
            &tree,
            &mut output,
            &mut stats,
            params,
            2,
            None,
            false,
            false,
            None,
        );
        assert!(result.is_err(), "expected error for oversized range");
        assert!(
            output.is_empty(),
            "no header should be written on validation failure, got: {:?}",
            String::from_utf8_lossy(&output)
        );
    }
}
