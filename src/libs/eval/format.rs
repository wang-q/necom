//! Output formatting helpers for clustering evaluation metrics.

use super::coordinates::Coordinates;
use super::distance::DistanceMatrix;
use super::pairwise::Metrics;
use super::LabelMap;
use super::{
    ball_hall_score, c_index_score, calinski_harabasz_score, davies_bouldin_score,
    dunn_score, gamma_score, pbm_score, silhouette_score, tau_score,
    wemmert_gancarski_score, xie_beni_score,
};
use std::fmt::Write as _;

/// External (pairwise) evaluation metric names, in output column order.
pub const EXTERNAL_METRIC_NAMES: &[&str] = &[
    "ari",
    "ami",
    "homogeneity",
    "completeness",
    "v_measure",
    "fmi",
    "nmi",
    "mi",
    "ri",
    "jaccard",
    "precision",
    "recall",
];

/// Distance-based evaluation metric names, in output column order.
pub const DISTANCE_METRIC_NAMES: &[&str] =
    &["silhouette", "dunn", "c_index", "gamma", "tau"];

/// Coordinate-based evaluation metric names, in output column order.
pub const COORD_METRIC_NAMES: &[&str] = &[
    "davies_bouldin",
    "calinski_harabasz",
    "pbm",
    "ball_hall",
    "xie_beni",
    "wemmert_gancarski",
];

/// External metric values from a Metrics struct, in EXTERNAL_METRIC_NAMES order.
pub fn external_metric_values(m: &Metrics) -> Vec<f64> {
    vec![
        m.ari,
        m.ami,
        m.homogeneity,
        m.completeness,
        m.v_measure,
        m.fmi,
        m.nmi,
        m.mi,
        m.ri,
        m.jaccard,
        m.precision,
        m.recall,
    ]
}

/// Distance-based metric values, in DISTANCE_METRIC_NAMES order.
pub fn distance_metric_values(
    partition: &LabelMap,
    dist_mat: &dyn DistanceMatrix,
) -> Vec<f64> {
    vec![
        silhouette_score(partition, dist_mat),
        dunn_score(partition, dist_mat),
        c_index_score(partition, dist_mat),
        gamma_score(partition, dist_mat),
        tau_score(partition, dist_mat),
    ]
}

/// Coordinate-based metric values, in COORD_METRIC_NAMES order.
pub fn coord_metric_values(partition: &LabelMap, coords: &Coordinates) -> Vec<f64> {
    vec![
        davies_bouldin_score(partition, coords),
        calinski_harabasz_score(partition, coords),
        pbm_score(partition, coords),
        ball_hall_score(partition, coords),
        xie_beni_score(partition, coords),
        wemmert_gancarski_score(partition, coords),
    ]
}

/// Format a slice of f64 values as tab-separated `{:.6}` strings.
///
/// Non-finite values (`NaN`, `+Infinity`, `-Infinity`) are emitted as `NA`
/// so that downstream TSV consumers (pandas, awk, R) can parse the column
/// without encountering bare `NaN`/`inf` literals. The metrics functions in
/// this module return `f64` (not `Result`), and signal degenerate inputs
/// (e.g., single-cluster silhouette, zero within-cluster variance in CH)
/// via `NaN` or `Infinity`; this formatter normalizes them to `NA`.
pub fn format_metrics_row(values: &[f64]) -> String {
    let mut out = String::with_capacity(values.len() * 16);
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            out.push('\t');
        }
        if v.is_finite() {
            // Writing to a String never fails; ignore the Result.
            let _ = write!(out, "{:.6}", v);
        } else {
            out.push_str("NA");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_metrics_row_basic() {
        let values = vec![1.0, 2.5, 4.5678];
        assert_eq!(format_metrics_row(&values), "1.000000\t2.500000\t4.567800");
    }

    #[test]
    fn test_format_metrics_row_empty() {
        let values: Vec<f64> = vec![];
        assert_eq!(format_metrics_row(&values), "");
    }

    #[test]
    fn test_format_metrics_row_negative_and_large() {
        let values = vec![-0.1234567, 1e9 + 0.5];
        assert_eq!(format_metrics_row(&values), "-0.123457\t1000000000.500000");
    }

    #[test]
    fn test_format_metrics_row_nan_inf() {
        let values = vec![f64::NAN, f64::INFINITY, 1.5, f64::NEG_INFINITY];
        assert_eq!(format_metrics_row(&values), "NA\tNA\t1.500000\tNA");
    }

    #[test]
    fn test_format_metrics_row_negative_zero() {
        // Rust's `{:.6}` preserves the sign of -0.0, emitting "-0.000000".
        // This is harmless for downstream TSV consumers (awk/pandas treat
        // -0.0 == 0.0 numerically) and consistent with C printf behavior.
        let values = vec![-0.0, 0.0];
        assert_eq!(format_metrics_row(&values), "-0.000000\t0.000000");
    }
}
