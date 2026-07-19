//! Medoid selection for flat clustering output.
//!
//! A medoid is the member of a cluster whose sum of pairwise distances
//! (or similarities) to all members (including itself) is extremal. For
//! distance matrices the medoid minimizes the sum; for similarity
//! matrices it maximizes the sum. Since self-distance / self-similarity
//! is constant across candidates, including it does not affect the
//! result. Ties are broken by the iteration order — callers should pass
//! `members` sorted by name so the alphabetically-first member wins.

use crate::libs::pairmat::MatrixView;

/// Find the medoid of a cluster.
///
/// Iterates `members` and returns the index of the member whose sum of
/// `matrix.get(candidate, member)` over all `members` is minimal
/// (`find_max = false`, distance matrix) or maximal (`find_max = true`,
/// similarity matrix). Returns `None` for an empty `members` slice.
///
/// Tie-breaking: the first member achieving the extremal sum wins, so
/// callers should sort `members` by name beforehand for deterministic
/// alphabetical tie-breaking.
pub fn find_medoid<M>(matrix: &M, members: &[usize], find_max: bool) -> Option<usize>
where
    M: MatrixView<f32>,
{
    if members.is_empty() {
        return None;
    }
    let mut best_rep = members[0];
    let mut best_sum = if find_max {
        f32::NEG_INFINITY
    } else {
        f32::MAX
    };

    for &candidate in members {
        let mut current_sum = 0.0;
        for &member in members {
            current_sum += matrix.get(candidate, member);
        }
        if find_max {
            if current_sum > best_sum {
                best_sum = current_sum;
                best_rep = candidate;
            }
        } else if current_sum < best_sum {
            best_sum = current_sum;
            best_rep = candidate;
        }
    }

    Some(best_rep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::pairmat::ScoringMatrix;

    fn build_matrix(size: usize, default: f32) -> ScoringMatrix<f32> {
        ScoringMatrix::<f32>::with_size_and_defaults(size, 0.0, default)
    }

    #[test]
    fn test_find_medoid_empty() {
        let m = build_matrix(3, 0.0);
        assert!(find_medoid(&m, &[], false).is_none());
        assert!(find_medoid(&m, &[], true).is_none());
    }

    #[test]
    fn test_find_medoid_single() {
        let m = build_matrix(3, 0.0);
        // The only member is index 2; it must be returned regardless of mode.
        assert_eq!(find_medoid(&m, &[2], false), Some(2));
        assert_eq!(find_medoid(&m, &[2], true), Some(2));
    }

    #[test]
    fn test_find_medoid_distance_matrix() {
        // 3 points; pairwise distances:
        //   0-1: 1.0, 0-2: 4.0, 1-2: 3.0
        // Sum of distances:
        //   0 -> 0 + 1 + 4 = 5
        //   1 -> 1 + 0 + 3 = 4  (min)
        //   2 -> 4 + 3 + 0 = 7
        let mut m = build_matrix(3, 100.0);
        m.set(0, 1, 1.0);
        m.set(0, 2, 4.0);
        m.set(1, 2, 3.0);

        assert_eq!(find_medoid(&m, &[0, 1, 2], false), Some(1));
    }

    #[test]
    fn test_find_medoid_similarity_matrix() {
        // 3 points; pairwise similarities (self=1.0 by default):
        //   0-1: 0.5, 0-2: 0.1, 1-2: 0.4
        // Sum of similarities:
        //   0 -> 1.0 + 0.5 + 0.1 = 1.6
        //   1 -> 0.5 + 1.0 + 0.4 = 1.9  (max)
        //   2 -> 0.1 + 0.4 + 1.0 = 1.5
        let mut m = build_matrix(3, 0.0);
        m.set(0, 0, 1.0);
        m.set(1, 1, 1.0);
        m.set(2, 2, 1.0);
        m.set(0, 1, 0.5);
        m.set(0, 2, 0.1);
        m.set(1, 2, 0.4);

        assert_eq!(find_medoid(&m, &[0, 1, 2], true), Some(1));
    }

    #[test]
    fn test_find_medoid_tie_breaking() {
        // All pairwise distances are equal, so every candidate ties on the sum.
        // The first member in `members` must win.
        let mut m = build_matrix(3, 0.0);
        m.set(0, 1, 1.0);
        m.set(0, 2, 1.0);
        m.set(1, 2, 1.0);

        // members = [2, 0, 1]: the first member is 2, so 2 wins the tie.
        assert_eq!(find_medoid(&m, &[2, 0, 1], false), Some(2));
        // members = [0, 1, 2]: the first member is 0, so 0 wins the tie.
        assert_eq!(find_medoid(&m, &[0, 1, 2], false), Some(0));
    }
}
