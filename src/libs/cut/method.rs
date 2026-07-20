/// Cut strategies for tree partitioning.
#[derive(Debug, Clone, Copy)]
pub enum Method {
    /// Cut into exactly K clusters.
    ///
    /// Iteratively splits the cluster with the largest height (distance to farthest leaf).
    K(usize),

    /// Cut at specific height (distance from leaves).
    ///
    /// Useful for ultrametric trees where height represents time/divergence.
    Height(f64),

    /// Cut at specific distance from root.
    ///
    /// Useful for defining clusters based on divergence from a common ancestor (root).
    RootDist(f64),

    /// TreeCluster: Max pairwise distance in clade <= threshold.
    ///
    /// Ensures that for every cluster, the maximum distance between any two leaves
    /// in that cluster is at most `threshold`.
    MaxClade(f64),

    /// TreeCluster: Average pairwise distance in clade <= threshold.
    AvgClade(f64),

    /// TreeCluster: Median pairwise distance in clade <= threshold.
    MedClade(f64),

    /// TreeCluster: Sum of branch lengths in clade <= threshold.
    SumBranch(f64),

    /// SciPy: Inconsistent coefficient <= threshold.
    ///
    /// Splits nodes if their inconsistency coefficient > threshold.
    /// Requires checking inconsistency of all descendants.
    /// Parameters: (threshold, depth).
    Inconsistent(f64, usize),

    /// TreeCluster: Single Linkage.
    ///
    /// Removes any edge (branch) with length > threshold.
    /// The resulting connected components (subtrees) form clusters.
    /// Note: This is equivalent to `Height` on ultrametric trees but generalizes to any tree.
    /// It effectively breaks "long branches".
    SingleLinkage(f64),
}

/// Supported cut method names, in detection priority order.
/// Excludes `dynamic-tree` and `dynamic-hybrid` which are handled separately.
pub const METHOD_NAMES: &[&str] = &[
    "k",
    "height",
    "root_dist",
    "max_clade",
    "avg_clade",
    "med_clade",
    "sum_branch",
    "leaf_dist_max",
    "leaf_dist_min",
    "leaf_dist_avg",
    "max_edge",
    "inconsistent",
];

/// Build a Method from a name and threshold value.
///
/// For "leaf-dist-*" methods, `leaf_depths` must be provided as `(min, max, avg)`.
pub fn build_method(
    name: &str,
    val: f64,
    deep: usize,
    leaf_depths: Option<(f64, f64, f64)>,
) -> anyhow::Result<Method> {
    let require_non_negative = |v: f64, method: &str| -> anyhow::Result<()> {
        if !v.is_finite() || v < 0.0 {
            anyhow::bail!(
                "{} threshold must be a non-negative finite number, got {}",
                method,
                v
            );
        }
        Ok(())
    };

    match name {
        "k" => {
            if val < 1.0 || val.fract() != 0.0 {
                anyhow::bail!("k must be a positive integer, got {}", val);
            }
            Ok(Method::K(val as usize))
        }
        "height" => {
            require_non_negative(val, "height")?;
            Ok(Method::Height(val))
        }
        "root_dist" => {
            require_non_negative(val, "root-dist")?;
            Ok(Method::RootDist(val))
        }
        "max_clade" => {
            require_non_negative(val, "max-clade")?;
            Ok(Method::MaxClade(val))
        }
        "avg_clade" => {
            require_non_negative(val, "avg-clade")?;
            Ok(Method::AvgClade(val))
        }
        "med_clade" => {
            require_non_negative(val, "med-clade")?;
            Ok(Method::MedClade(val))
        }
        "sum_branch" => {
            require_non_negative(val, "sum-branch")?;
            Ok(Method::SumBranch(val))
        }
        "leaf_dist_max" => {
            require_non_negative(val, "leaf-dist-max")?;
            match leaf_depths {
                Some((_, max, _)) => {
                    if val > max {
                        anyhow::bail!(
                            "--leaf-dist-max threshold {} exceeds maximum leaf depth {}",
                            val,
                            max
                        );
                    }
                    Ok(Method::RootDist(max - val))
                }
                None => Err(anyhow::anyhow!("leaf depths required for leaf-dist-max")),
            }
        }
        "leaf_dist_min" => {
            require_non_negative(val, "leaf-dist-min")?;
            match leaf_depths {
                Some((min, _, _)) => {
                    if val > min {
                        anyhow::bail!(
                            "--leaf-dist-min threshold {} exceeds minimum leaf depth {}",
                            val,
                            min
                        );
                    }
                    Ok(Method::RootDist(min - val))
                }
                None => Err(anyhow::anyhow!("leaf depths required for leaf-dist-min")),
            }
        }
        "leaf_dist_avg" => {
            require_non_negative(val, "leaf-dist-avg")?;
            match leaf_depths {
                Some((_, _, avg)) => {
                    if val > avg {
                        anyhow::bail!(
                            "--leaf-dist-avg threshold {} exceeds average leaf depth {}",
                            val,
                            avg
                        );
                    }
                    Ok(Method::RootDist(avg - val))
                }
                None => Err(anyhow::anyhow!("leaf depths required for leaf-dist-avg")),
            }
        }
        "max_edge" => {
            require_non_negative(val, "max-edge")?;
            Ok(Method::SingleLinkage(val))
        }
        "inconsistent" => {
            require_non_negative(val, "inconsistent")?;
            Ok(Method::Inconsistent(val, deep))
        }
        _ => anyhow::bail!("unknown method: {}", name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_method_negative_threshold_rejected() {
        // All threshold-based methods should reject negative values.
        let threshold_methods = [
            "height",
            "root_dist",
            "max_clade",
            "avg_clade",
            "med_clade",
            "sum_branch",
            "leaf_dist_max",
            "leaf_dist_min",
            "leaf_dist_avg",
            "max_edge",
            "inconsistent",
        ];
        for name in threshold_methods {
            let result = build_method(name, -1.0, 2, Some((1.0, 3.0, 2.0)));
            assert!(
                result.is_err(),
                "method {} should reject negative threshold",
                name
            );
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.contains("non-negative"),
                "error message should mention non-negative: {}",
                msg
            );
        }
    }

    #[test]
    fn test_build_method_zero_threshold_allowed() {
        // Zero is a valid threshold for distance/height methods.
        assert!(build_method("height", 0.0, 2, None).is_ok());
        assert!(build_method("max_clade", 0.0, 2, None).is_ok());
    }

    #[test]
    fn test_build_method_non_finite_threshold_rejected() {
        for (value, label) in [
            (f64::NAN, "NaN"),
            (f64::INFINITY, "+Inf"),
            (f64::NEG_INFINITY, "-Inf"),
        ] {
            let result = build_method("height", value, 2, None);
            assert!(result.is_err(), "height should reject {}", label);
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.contains("non-negative finite"),
                "error message should mention non-negative finite: {}",
                msg
            );
        }
    }

    #[test]
    fn test_build_method_k_rejects_non_positive() {
        assert!(build_method("k", 0.0, 2, None).is_err());
        assert!(build_method("k", -1.0, 2, None).is_err());
        assert!(build_method("k", 1.5, 2, None).is_err());
        assert!(build_method("k", 2.0, 2, None).is_ok());
    }

    #[test]
    fn test_build_method_leaf_dist_rejects_threshold_exceeding_depth() {
        // leaf_depths = (min=1.0, max=3.0, avg=2.0).
        let depths = Some((1.0, 3.0, 2.0));
        assert!(build_method("leaf_dist_max", 3.0, 2, depths).is_ok());
        assert!(build_method("leaf_dist_max", 3.1, 2, depths).is_err());
        assert!(build_method("leaf_dist_min", 1.0, 2, depths).is_ok());
        assert!(build_method("leaf_dist_min", 1.1, 2, depths).is_err());
        assert!(build_method("leaf_dist_avg", 2.0, 2, depths).is_ok());
        assert!(build_method("leaf_dist_avg", 2.1, 2, depths).is_err());
    }
}
