#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::NecomCmd;
use std::fmt::Write as _;
use std::fs;

// --- Helper: Generate Synthetic Blobs ---
// Generates 3 well-separated clusters in 2D space.
// Format: ID \t X,Y
// Ground Truth: ID \t ClusterID
fn generate_blobs(
    data_file: &str,
    truth_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut data_out = String::new();
    let mut truth_out = String::new();

    // Cluster 1: Center (0,0), 10 points
    for i in 0..10 {
        let x = 0.0 + (i as f64 * 0.1);
        let y = 0.0 + (i as f64 * 0.1);
        let id = format!("C1_{}", i);
        writeln!(data_out, "{}\t{:.4},{:.4}", id, x, y)?;
        writeln!(truth_out, "1\t{}", id)?;
    }

    // Cluster 2: Center (10,10), 10 points
    for i in 0..10 {
        let x = 10.0 + (i as f64 * 0.1);
        let y = 10.0 + (i as f64 * 0.1);
        let id = format!("C2_{}", i);
        writeln!(data_out, "{}\t{:.4},{:.4}", id, x, y)?;
        writeln!(truth_out, "2\t{}", id)?;
    }

    // Cluster 3: Center (20,0), 10 points
    for i in 0..10 {
        let x = 20.0 + (i as f64 * 0.1);
        let y = 0.0 + (i as f64 * 0.1);
        let id = format!("C3_{}", i);
        writeln!(data_out, "{}\t{:.4},{:.4}", id, x, y)?;
        writeln!(truth_out, "3\t{}", id)?;
    }

    fs::write(data_file, data_out)?;
    fs::write(truth_file, truth_out)?;
    Ok(())
}

// Compute Euclidean pairwise distances from the coordinate TSV produced by
// `generate_blobs` and write them in the three-column TSV format expected by
// `necom mat to-phylip`.
fn compute_pairwise_distances(
    data_file: &str,
    dist_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(data_file)?;
    let mut points: Vec<(String, f64, f64)> = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 2 {
            continue;
        }
        let coords: Vec<&str> = parts[1].split(',').collect();
        if coords.len() != 2 {
            continue;
        }
        let x: f64 = coords[0].parse()?;
        let y: f64 = coords[1].parse()?;
        points.push((parts[0].to_string(), x, y));
    }

    let mut out = String::new();
    for (id1, x1, y1) in &points {
        for (id2, x2, y2) in &points {
            let dist = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
            writeln!(out, "{}\t{}\t{:.6}", id1, id2, dist)?;
        }
    }

    fs::write(dist_file, out)?;
    Ok(())
}

#[test]
fn test_clust_pipeline_full() {
    let temp_dir = tempfile::Builder::new()
        .prefix("necom_pipeline_test")
        .tempdir()
        .expect("Failed to create temp dir");
    let base_dir = temp_dir.path().to_str().unwrap();

    let data_file = format!("{}/blobs.tsv", base_dir);
    let truth_file = format!("{}/blobs.truth.tsv", base_dir);

    // Intermediate files
    let dist_file = format!("{}/blobs.dist.tsv", base_dir);
    let phy_file = format!("{}/blobs.phy", base_dir);
    let tree_file = format!("{}/blobs.nwk", base_dir);
    let cut_file = format!("{}/blobs.cut.tsv", base_dir);
    let _eval_file = format!("{}/blobs.eval.tsv", base_dir);

    // 1. Generate Data
    generate_blobs(&data_file, &truth_file).expect("Failed to generate data");

    // 2. Calculate Euclidean pairwise distances
    // Input: ID \t X,Y
    // Output: ID1 \t ID2 \t Dist
    compute_pairwise_distances(&data_file, &dist_file)
        .expect("Failed to compute distances");
    assert!(fs::metadata(&dist_file).is_ok(), "dist file not created");

    // 3. Convert to PHYLIP Matrix (necom mat to-phylip)
    let (_stdout, stderr) = NecomCmd::new()
        .args(&["mat", "to-phylip", &dist_file, "-o", &phy_file])
        .run();
    assert!(stderr.is_empty(), "mat to-phylip failed: {}", stderr);
    assert!(fs::metadata(&phy_file).is_ok(), "phylip file not created");

    // 4. Hierarchical Clustering (necom clust hier)
    // Method: Ward (standard for Euclidean)
    let (_stdout, stderr) = NecomCmd::new()
        .args(&[
            "clust", "hier", &phy_file, "--method", "ward", "-o",
            &tree_file, // Explicit output file
        ])
        .run();
    assert!(stderr.is_empty(), "clust hier failed: {}", stderr);
    assert!(fs::metadata(&tree_file).is_ok(), "tree file not created");

    // 5. Cut Tree (necom cut)
    // We know there are 3 clusters, so use --k 3
    let (_stdout, stderr) = NecomCmd::new()
        .args(&[
            "cut",
            "simple",
            &tree_file,
            "--method",
            "k",
            "--threshold",
            "3",
            "--format",
            "pair", // Output: Rep \t Member (compatible with eval)
            "-o",
            &cut_file,
        ])
        .run();
    assert!(stderr.is_empty(), "cut failed: {}", stderr);
    assert!(fs::metadata(&cut_file).is_ok(), "cut file not created");

    // 6. Evaluate (necom eval partition)
    // Compare cut result with ground truth
    // Output ARI should be 1.0 for perfect clustering
    let (stdout, stderr) = NecomCmd::new()
        .args(&[
            "eval",
            "partition",
            &cut_file, // Prediction
            "--other",
            &truth_file, // Ground Truth
            "--input-format",
            "pair", // Both are in pair format (or at least compatible)
        ])
        .run();

    assert!(stderr.is_empty(), "eval partition failed: {}", stderr);

    let ari_line = stdout
        .lines()
        .find(|l| l.trim().starts_with("0.") || l.trim().starts_with("1."))
        .expect("Score line not found");
    let parts: Vec<&str> = ari_line.split_whitespace().collect();
    let ari_val: f64 = parts[0].parse().expect("Failed to parse ARI"); // ARI is first column

    assert!(
        (ari_val - 1.0).abs() < 1e-4,
        "ARI should be 1.0, got {}",
        ari_val
    );

    // Cleanup handled by tempdir drop, or we can explicit close if needed but Drop trait handles it.
}
