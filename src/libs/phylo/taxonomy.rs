//! Taxonomy TSV parsing for tree condensation pipelines.

use std::collections::{BTreeMap, BTreeSet};
use std::io::BufRead;

use super::parser::newick_safe;

/// Taxonomy table: per-node terms grouped by rank, plus unique groups per rank.
#[derive(Debug, Default)]
pub struct TaxonomyTable {
    /// node_name -> terms per rank (None if column missing or empty).
    pub taxon_map: BTreeMap<String, Vec<Option<String>>>,
    /// Unique, sorted, NA-filtered group names per rank.
    pub all_groups: Vec<Vec<String>>,
}

/// Read taxonomy TSV from `reader` and filter to `leaf_names`.
///
/// `ranks` is a list of 1-based column indices into the TSV. Lines with fewer
/// than 2 columns are skipped with a warning. Nodes not in `leaf_names` are
/// ignored. Group lists are deduplicated, sorted, and filtered to drop `"NA"`.
pub fn read_taxonomy<R: BufRead>(
    reader: R,
    ranks: &[usize],
    leaf_names: &BTreeSet<String>,
) -> anyhow::Result<TaxonomyTable> {
    // Pre-validate rank indices once (defense-in-depth; caller already checks).
    // Hoisting this out of the per-line loop avoids N_lines x N_ranks redundant
    // checks on large taxonomy TSVs.
    for rank_col in ranks {
        if *rank_col == 0 {
            anyhow::bail!("rank column index must be 1-based");
        }
    }

    let mut taxon_map: BTreeMap<String, Vec<Option<String>>> = BTreeMap::new();
    let mut all_groups: Vec<Vec<String>> = vec![vec![]; ranks.len()];

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            log::warn!("skipping line with <2 columns: {}", line);
            continue;
        }
        let node_name = parts[0].to_string();
        if !leaf_names.contains(&node_name) {
            continue;
        }
        let mut terms: Vec<Option<String>> = Vec::with_capacity(ranks.len());

        for (i, rank_col) in ranks.iter().enumerate() {
            let rank_idx = rank_col - 1;
            // Treat missing, empty, or whitespace-only cells as None, matching
            // the documented "None if column missing or empty" contract.
            let term = parts.get(rank_idx).and_then(|s| {
                if s.trim().is_empty() {
                    None
                } else {
                    Some(newick_safe(s))
                }
            });
            if let Some(t) = &term {
                all_groups[i].push(t.clone());
            }
            terms.push(term);
        }

        if terms.iter().any(|t| t.is_some()) {
            taxon_map.insert(node_name, terms);
        }
    }

    // Deduplicate, sort, and filter NA per rank.
    for groups in &mut all_groups {
        groups.sort();
        groups.dedup();
        groups.retain(|s| s != "NA");
    }

    Ok(TaxonomyTable {
        taxon_map,
        all_groups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::io::Cursor;

    fn leaf_set(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_read_taxonomy_rejects_zero_rank() {
        // rank column index 0 is invalid (1-based); must error before reading lines.
        let tsv = "A\tg1\nB\tg2\n";
        let leaves = leaf_set(&["A", "B"]);
        let result = read_taxonomy(Cursor::new(tsv), &[0], &leaves);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("1-based"),
            "expected 1-based hint in error, got: {}",
            msg
        );
    }

    #[test]
    fn test_read_taxonomy_zero_rank_errors_without_reading_lines() {
        // Even with an empty TSV, rank 0 must be rejected.
        let leaves = leaf_set(&[]);
        let result = read_taxonomy(Cursor::new(""), &[2, 0], &leaves);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_taxonomy_basic_grouping() {
        let tsv = "A\tsp1\tgen1\nB\tsp1\tgen1\nC\tsp2\tgen2\n";
        let leaves = leaf_set(&["A", "B", "C"]);
        let table = read_taxonomy(Cursor::new(tsv), &[2], &leaves).unwrap();
        // all_groups[0] for rank=2 should contain sp1, sp2 (sorted, deduped).
        assert_eq!(
            table.all_groups[0],
            vec!["sp1".to_string(), "sp2".to_string()]
        );
        // taxon_map should have all three leaves.
        assert_eq!(table.taxon_map.len(), 3);
    }

    #[test]
    fn test_read_taxonomy_filters_non_leaf_names() {
        let tsv = "A\tg1\nX\tg2\nB\tg3\n";
        let leaves = leaf_set(&["A", "B"]);
        let table = read_taxonomy(Cursor::new(tsv), &[2], &leaves).unwrap();
        // X is not in leaf_names, must be excluded.
        assert!(!table.taxon_map.contains_key("X"));
        assert_eq!(table.taxon_map.len(), 2);
    }

    #[test]
    fn test_read_taxonomy_skips_lines_with_fewer_columns() {
        let tsv = "A\tg1\nloneline\nB\tg2\n";
        let leaves = leaf_set(&["A", "B", "loneline"]);
        let table = read_taxonomy(Cursor::new(tsv), &[2], &leaves).unwrap();
        // loneline has <2 columns, skipped; not in taxon_map.
        assert!(!table.taxon_map.contains_key("loneline"));
        assert_eq!(table.taxon_map.len(), 2);
    }

    #[test]
    fn test_read_taxonomy_filters_na_groups() {
        let tsv = "A\tNA\tg1\nB\tsp1\tg1\n";
        let leaves = leaf_set(&["A", "B"]);
        let table = read_taxonomy(Cursor::new(tsv), &[2, 3], &leaves).unwrap();
        // all_groups[0] (rank=2) should not contain "NA".
        assert!(!table.all_groups[0].contains(&"NA".to_string()));
        // A should still be in taxon_map (has a term at rank 3).
        assert!(table.taxon_map.contains_key("A"));
    }

    #[test]
    fn test_read_taxonomy_empty_cell_becomes_none() {
        let tsv = "A\t\tg1\n";
        let leaves = leaf_set(&["A"]);
        let table = read_taxonomy(Cursor::new(tsv), &[2, 3], &leaves).unwrap();
        let terms = table.taxon_map.get("A").unwrap();
        // Rank 2 (column 2) is empty -> None; rank 3 (column 3) is "g1".
        assert_eq!(terms.len(), 2);
        assert!(terms[0].is_none());
        assert_eq!(terms[1].as_deref(), Some("g1"));
    }
}
