//! Flat clustering output formatting.
//!
//! Shared formatting logic for clustering algorithms that produce
//! `Vec<Vec<usize>>` results (DBSCAN, K-Medoids, MCL, Connected Components).

use std::fmt::Write as _;

/// Sort and format flat clustering results (indices into `names`).
///
/// Members within each cluster are sorted alphabetically by name; clusters
/// are sorted by size (descending) then by first member name. `rep_fn`
/// selects the representative index for each cluster. For "pair" format,
/// returning `None` skips that cluster. For "cluster" format, the
/// representative is placed in the first column if one is returned.
pub fn format_flat_clusters<F>(
    clusters: &mut Vec<Vec<usize>>,
    names: &[String],
    format: &str,
    rep_fn: F,
) -> anyhow::Result<String>
where
    F: Fn(&[usize]) -> Option<usize>,
{
    // Sort members within each cluster alphabetically by name.
    for c in clusters.iter_mut() {
        c.sort_by_key(|&idx| &names[idx]);
    }
    // Sort clusters: size desc, then first member name.
    clusters.sort_by(|a, b| match b.len().cmp(&a.len()) {
        std::cmp::Ordering::Equal => names[a[0]].cmp(&names[b[0]]),
        other => other,
    });

    // Rough capacity estimate: one byte per character is a lower bound; the
    // multiplier accounts for tabs/newlines and typical name lengths.
    let total_members: usize = clusters.iter().map(|c| c.len()).sum();
    let mut out = String::with_capacity(total_members * 16);

    match format {
        "cluster" => {
            for c in clusters {
                let rep_idx = rep_fn(c);
                let mut members: Vec<&str> =
                    c.iter().map(|&idx| names[idx].as_str()).collect();
                if let Some(rep) = rep_idx {
                    // Move the representative to the first column.
                    if let Some(pos) = c.iter().position(|&idx| idx == rep) {
                        if pos > 0 {
                            let rep_name = members.remove(pos);
                            members.insert(0, rep_name);
                        }
                    }
                }
                for (i, name) in members.iter().enumerate() {
                    if i > 0 {
                        write!(out, "\t")?;
                    }
                    write!(out, "{}", name)?;
                }
                writeln!(out)?;
            }
        }
        "pair" => {
            for c in clusters.iter() {
                if let Some(rep_idx) = rep_fn(c) {
                    let rep_name = &names[rep_idx];
                    for &member_idx in c {
                        writeln!(out, "{}\t{}", rep_name, names[member_idx])?;
                    }
                }
            }
        }
        _ => anyhow::bail!("unsupported output format: {}", format),
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_flat_clusters_rep_at_last_position() {
        // Members are sorted alphabetically by name: A(0), B(1), C(2).
        // Choose C as representative so it starts at the last position.
        let mut clusters = vec![vec![2, 1, 0]];
        let names = vec!["A".to_string(), "B".to_string(), "C".to_string()];

        let out = format_flat_clusters(&mut clusters, &names, "cluster", |c| {
            c.iter().find(|&&idx| idx == 2).copied()
        })
        .unwrap();

        assert_eq!(out, "C\tA\tB\n");
    }

    #[test]
    fn test_format_flat_clusters_pair() {
        let mut clusters = vec![vec![0, 1]];
        let names = vec!["A".to_string(), "B".to_string()];

        let out =
            format_flat_clusters(&mut clusters, &names, "pair", |c| c.first().copied())
                .unwrap();

        assert_eq!(out, "A\tA\nA\tB\n");
    }

    #[test]
    fn test_format_flat_clusters_unsupported_format() {
        let mut clusters = Vec::new();
        let names = Vec::new();
        assert!(
            format_flat_clusters(&mut clusters, &names, "unknown", |_| None).is_err()
        );
    }
}
