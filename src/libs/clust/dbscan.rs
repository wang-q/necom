//! Implementation of the [DBSCAN](https://en.wikipedia.org/wiki/DBSCAN) clustering algorithm.
//!
//! Key features:
//! * Density-based clustering
//! * Automatic noise detection
//! * No predefined cluster count
//! * Handles arbitrary cluster shapes
//!
//! Parameters:
//! * eps: Neighborhood radius
//! * min_points: Core point threshold
//!
//! Output formats:
//! * Cluster labels: Some(id) or None (noise)
//! * Cluster groups: Vec<Vec<point_indices>>
//!
//! Adapted from <https://blog.petrzemek.net/2017/01/01/implementing-dbscan-from-distance-matrix-in-rust/>.
use std::collections::{HashMap, VecDeque};

/// DBSCAN clustering from a distance matrix.
#[derive(Debug)]
pub struct Dbscan<T> {
    /// Maximum distance between two points to be considered neighbors.
    eps: T,
    /// Minimum number of points (including the point itself) to form a dense region.
    min_points: usize,
    /// Cluster label for each point; `Some(id)` for cluster members, `None` for noise.
    clusters: Vec<Option<usize>>,
    /// Whether each point has been visited by the algorithm.
    visited: Vec<bool>,
    /// Next cluster identifier to assign.
    current_cluster: usize,
}

impl<T> Dbscan<T>
where
    T: Default + Copy + PartialOrd,
{
    /// Creates a new DBSCAN instance.
    ///
    /// # Parameters
    ///
    /// * `eps` - The maximum distance between two points for them to be in the
    ///   same neighborhood. Must be positive.
    /// * `min_points` - The minimal number of points in a neighborhood for a
    ///   point to be considered as a core point. Must be at least 1.
    ///
    /// # Errors
    ///
    /// Returns an error if `eps` is non-positive or NaN, or `min_points` is zero.
    pub fn new(eps: T, min_points: usize) -> anyhow::Result<Self> {
        // `partial_cmp` returns `None` when either operand is NaN, so checking
        // for `Some(Greater)` rejects non-positive values *and* NaN. Using
        // `eps <= T::default()` alone would let NaN through (`NaN <= 0.0` is
        // false under `PartialOrd`).
        if eps.partial_cmp(&T::default()) != Some(std::cmp::Ordering::Greater) {
            anyhow::bail!("eps must be a positive number");
        }
        if min_points == 0 {
            anyhow::bail!("min_points must be at least 1");
        }
        Ok(Dbscan {
            eps,
            min_points,
            clusters: Vec::new(),
            visited: Vec::new(),
            current_cluster: 0,
        })
    }

    /// Performs DBSCAN clustering from the given distance matrix.
    ///
    /// # Returns
    ///
    /// Returns a reference to the vector of cluster labels for each point in the dataset.
    /// * `Some(id)`: The point belongs to cluster `id`.
    /// * `None`: The point is considered noise.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use necom::libs::clust::dbscan::Dbscan;
    /// # use necom::libs::pairmat::ScoringMatrix;
    ///
    /// let mut dbscan = Dbscan::new(1, 2).unwrap();
    /// let mut m = ScoringMatrix::<i8>::with_size_and_defaults(5, 0, 100);
    /// m.set(0, 1, 1);
    /// m.set(0, 2, 9);
    /// m.set(0, 3, 9);
    /// m.set(0, 4, 9);
    /// m.set(1, 2, 9);
    /// m.set(1, 3, 9);
    /// m.set(1, 4, 9);
    /// m.set(2, 3, 1);
    /// m.set(2, 4, 9);
    /// m.set(3, 4, 9);
    ///
    /// let clustering = dbscan.perform_clustering(&m);
    ///
    /// assert_eq!(clustering[0], Some(0));
    /// assert_eq!(clustering[1], Some(0));
    /// assert_eq!(clustering[2], Some(1));
    /// assert_eq!(clustering[3], Some(1));
    /// assert_eq!(clustering[4], None);
    /// ```
    ///
    /// In the above example, points `0` and `1` form a single cluster, points
    /// `2` and `3` form a different cluster, and point `4` does not belong to any
    /// cluster (it is a noise point).
    pub fn perform_clustering<M>(&mut self, matrix: &M) -> &Vec<Option<usize>>
    where
        M: crate::libs::pairmat::MatrixView<T>,
    {
        self.clusters = vec![None; matrix.size()];
        self.visited = vec![false; matrix.size()];
        self.current_cluster = 0;

        for point in 0..matrix.size() {
            if self.visited[point] {
                continue;
            }

            self.visited[point] = true;
            let neighbors = self.region_query(matrix, point);
            if neighbors.len() >= self.min_points {
                self.expand_cluster(matrix, point, neighbors);
                self.current_cluster += 1;
            }
        }

        self.clusters.as_ref()
    }

    fn expand_cluster<M>(
        &mut self,
        matrix: &M,
        point: usize,
        mut neighbors: VecDeque<usize>,
    ) where
        M: crate::libs::pairmat::MatrixView<T>,
    {
        self.clusters[point] = Some(self.current_cluster);

        while let Some(other_point) = neighbors.pop_front() {
            if !self.visited[other_point] {
                self.visited[other_point] = true;
                let mut other_neighbors = self.region_query(matrix, other_point);
                if other_neighbors.len() >= self.min_points {
                    neighbors.append(&mut other_neighbors);
                }
            }
            if self.clusters[other_point].is_none() {
                self.clusters[other_point] = Some(self.current_cluster);
            }
        }
    }

    fn region_query<M>(&self, matrix: &M, point: usize) -> VecDeque<usize>
    where
        M: crate::libs::pairmat::MatrixView<T>,
    {
        let mut neighbors = VecDeque::new();
        for other_point in 0..matrix.size() {
            let dist = matrix.get(point, other_point);
            if dist <= self.eps {
                neighbors.push_back(other_point);
            }
        }
        neighbors
    }

    fn all_clusters(&self) -> (HashMap<usize, Vec<usize>>, Vec<usize>) {
        let mut cluster_map: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut noise_points: Vec<usize> = Vec::new();

        for (point, cluster) in self.clusters.iter().enumerate() {
            match cluster {
                Some(cluster_id) => {
                    cluster_map.entry(*cluster_id).or_default().push(point);
                }
                None => {
                    noise_points.push(point);
                }
            }
        }
        (cluster_map, noise_points)
    }

    /// Returns the number of density clusters (non-noise) and the number of
    /// noise points.
    pub fn counts(&self) -> (usize, usize) {
        let mut cluster_ids = std::collections::HashSet::new();
        let mut noise = 0;
        for label in &self.clusters {
            match label {
                Some(id) => {
                    cluster_ids.insert(*id);
                }
                None => noise += 1,
            }
        }
        (cluster_ids.len(), noise)
    }

    /// Returns clusters as a vector of member-index vectors.
    ///
    /// Noise points (points not assigned to any density cluster) are returned
    /// as one-member clusters, consistent with the flat-clustering output
    /// convention used by the other `clust` commands.
    pub fn results_cluster(&self) -> Vec<Vec<usize>> {
        let (cluster_map, noise_points) = self.all_clusters();
        let mut res: Vec<Vec<usize>> = Vec::new();

        // Sort by cluster ID for deterministic output.
        let mut cluster_ids: Vec<usize> = cluster_map.keys().copied().collect();
        cluster_ids.sort();
        for id in cluster_ids {
            res.push(cluster_map[&id].clone());
        }
        for p in noise_points {
            res.push(vec![p]);
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_points_are_in_single_cluster_when_their_distance_is_zero() {
        let mut dbscan = Dbscan::new(1, 2).unwrap();
        let m =
            crate::libs::pairmat::ScoringMatrix::<i8>::with_size_and_defaults(2, 0, 1);

        let clustering = dbscan.perform_clustering(&m);

        assert_eq!(clustering[0], Some(0));
        assert_eq!(clustering[1], Some(0));
    }

    #[test]
    fn test_points_are_correctly_clustered_based_on_their_distance() {
        let mut dbscan = Dbscan::new(1, 2).unwrap();
        let mut m =
            crate::libs::pairmat::ScoringMatrix::<i8>::with_size_and_defaults(5, 0, 100);
        m.set(0, 1, 1);
        m.set(0, 2, 9);
        m.set(0, 3, 9);
        m.set(0, 4, 9);
        m.set(1, 2, 9);
        m.set(1, 3, 9);
        m.set(1, 4, 9);
        m.set(2, 3, 1);
        m.set(2, 4, 9);
        m.set(3, 4, 9);

        let clustering = dbscan.perform_clustering(&m);

        assert_eq!(clustering[0], Some(0));
        assert_eq!(clustering[1], Some(0));
        assert_eq!(clustering[2], Some(1));
        assert_eq!(clustering[3], Some(1));
        assert_eq!(clustering[4], None);
    }

    #[test]
    fn test_neighboring_points_are_put_into_cluster_even_if_they_have_been_visited() {
        // In 2D, the points in this test are placed as follows:
        //
        //    0
        //      1
        //        2
        //
        // Epsilon is set to 1 and min_points to 3. When the first point is
        // checked (0), it is marked as visited. Since it has only a single
        // neighbor, the two points (0 and 1) cannot form a cluster because
        // min_points is 3. Then, the algorithm continues to point 1. It has
        // two neighbors (0 and
        // 2), so the three points (0, 1, 2) can form a cluster. In this test,
        // we ensure that even when the first point (0) has already been
        // marked as visited, it is put into the cluster because it is not
        // yet a member of any other cluster.
        let mut dbscan = Dbscan::new(1, 3).unwrap();
        let mut m =
            crate::libs::pairmat::ScoringMatrix::<i8>::with_size_and_defaults(3, 0, 100);
        m.set(0, 1, 1);
        m.set(0, 2, 2);
        m.set(1, 2, 1);

        let clustering = dbscan.perform_clustering(&m);

        assert_eq!(clustering[0], Some(0));
        assert_eq!(clustering[1], Some(0));
        assert_eq!(clustering[2], Some(0));
    }

    #[test]
    fn test_points_that_do_not_belong_to_any_cluster_are_none() {
        let mut dbscan = Dbscan::new(1, 2).unwrap();
        let m =
            crate::libs::pairmat::ScoringMatrix::<i8>::with_size_and_defaults(1, 0, 100);

        let clustering = dbscan.perform_clustering(&m);

        assert_eq!(clustering[0], None);
    }

    #[test]
    fn test_self_distance_affects_core_point_status() {
        // Three points on a line with pairwise distances 1 and 2.
        // eps=1.5, min_points=3.
        // With self-distance 0, each point counts itself plus two others and
        // becomes a core point, so all points end up in one cluster.
        let mut dbscan = Dbscan::new(1.5_f32, 3).unwrap();
        let mut m = crate::libs::pairmat::ScoringMatrix::<f32>::with_size_and_defaults(
            3, 0.0, 100.0,
        );
        m.set(0, 1, 1.0);
        m.set(0, 2, 2.0);
        m.set(1, 2, 1.0);

        let clustering = dbscan.perform_clustering(&m);
        assert!(
            clustering.iter().all(|c| c.is_some()),
            "all points should be core with self-distance 0"
        );

        // With self-distance 2.0 (> eps), each point only sees two neighbors
        // and fails the min_points threshold, so no core points exist.
        let mut dbscan2 = Dbscan::new(1.5_f32, 3).unwrap();
        let mut m2 = crate::libs::pairmat::ScoringMatrix::<f32>::with_size_and_defaults(
            3, 2.0, 100.0,
        );
        m2.set(0, 1, 1.0);
        m2.set(0, 2, 2.0);
        m2.set(1, 2, 1.0);

        let clustering2 = dbscan2.perform_clustering(&m2);
        assert!(
            clustering2.iter().all(|c| c.is_none()),
            "no core points when self-distance exceeds eps"
        );
    }

    #[test]
    fn test_all_points_become_noise_then_singletons() {
        // Five points all separated by distances greater than eps=0.5.
        // None can become a core point, so every point is noise.
        // `results_cluster` must promote each noise point to its own
        // single-member cluster, matching the flat-clustering convention.
        let mut dbscan = Dbscan::new(0.5_f32, 2).unwrap();
        let mut m = crate::libs::pairmat::ScoringMatrix::<f32>::with_size_and_defaults(
            5, 0.0, 100.0,
        );
        // Pairwise distances all > eps.
        m.set(0, 1, 1.0);
        m.set(0, 2, 1.0);
        m.set(0, 3, 1.0);
        m.set(0, 4, 1.0);
        m.set(1, 2, 1.0);
        m.set(1, 3, 1.0);
        m.set(1, 4, 1.0);
        m.set(2, 3, 1.0);
        m.set(2, 4, 1.0);
        m.set(3, 4, 1.0);

        dbscan.perform_clustering(&m);
        let clusters = dbscan.results_cluster();

        // Each point becomes its own single-member cluster.
        assert_eq!(clusters.len(), 5);
        for c in &clusters {
            assert_eq!(c.len(), 1);
        }
        // Every original point index 0..5 must appear exactly once.
        let mut all_members: Vec<usize> = clusters.into_iter().flatten().collect();
        all_members.sort();
        assert_eq!(all_members, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_results_cluster_order_is_deterministic() {
        // Two well-separated clusters: {0,1} and {2,3}. Cluster IDs are assigned
        // in the order the algorithm discovers core points, so cluster 0 precedes
        // cluster 1. `results_cluster` must return clusters sorted by cluster ID
        // to keep the output deterministic regardless of HashMap iteration order.
        let mut dbscan = Dbscan::new(1.5_f32, 2).unwrap();
        let mut m = crate::libs::pairmat::ScoringMatrix::<f32>::with_size_and_defaults(
            4, 0.0, 10.0,
        );
        m.set(0, 1, 1.0);
        m.set(2, 3, 1.0);

        dbscan.perform_clustering(&m);
        let clusters = dbscan.results_cluster();

        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0], vec![0, 1]);
        assert_eq!(clusters[1], vec![2, 3]);
    }

    #[test]
    fn test_new_rejects_nan_eps() {
        // `NaN <= 0.0` is false under PartialOrd, so the old `eps <= T::default()`
        // check let NaN through. The `!(eps > T::default())` form rejects NaN
        // because `NaN > 0.0` is also false.
        let err = Dbscan::new(f32::NAN, 2).unwrap_err();
        assert!(
            err.to_string().contains("eps"),
            "error should mention eps, got: {}",
            err
        );
    }

    #[test]
    fn test_new_rejects_zero_and_negative_eps() {
        let err = Dbscan::new(0.0_f32, 2).unwrap_err();
        assert!(err.to_string().contains("eps"));

        let err = Dbscan::new(-1.0_f32, 2).unwrap_err();
        assert!(err.to_string().contains("eps"));
    }

    #[test]
    fn test_new_rejects_zero_min_points() {
        let err = Dbscan::new(1.0_f32, 0).unwrap_err();
        assert!(
            err.to_string().contains("min_points"),
            "error should mention min_points, got: {}",
            err
        );
    }
}
