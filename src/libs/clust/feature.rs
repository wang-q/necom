//! Feature vector for distance-based clustering.
//!
//! A `FeatureVector` pairs a name with a list of float coordinates, used
//! by `libs/clust/eval.rs` for distance
//! computation and cluster evaluation.

use anyhow::anyhow;
use std::io::BufRead;

//----------------------------
// FeatureVector
//----------------------------
/// A named feature vector for clustering input.
#[derive(Default, Clone)]
pub struct FeatureVector {
    name: String,
    list: Vec<f32>,
}

impl FeatureVector {
    // Immutable accessors
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn list(&self) -> &Vec<f32> {
        &self.list
    }

    pub fn new() -> Self {
        Self {
            name: String::new(),
            list: vec![],
        }
    }

    /// Constructed from range and seq
    ///
    /// ```ignore
    /// # use necom::libs::clust::feature::FeatureVector;
    /// let name = "Es_coli_005008_GCF_013426115_1".to_string();
    /// let list : Vec<f32> = vec![1.0,5.0,2.0,7.0,6.0,6.0];
    /// let entry = FeatureVector::from(&name, &list);
    /// # assert_eq!(*entry.name(), "Es_coli_005008_GCF_013426115_1");
    /// # assert_eq!(*entry.list().get(1).unwrap(), 5f32);
    /// ```
    pub fn from(name: &str, vector: &[f32]) -> Self {
        Self {
            name: name.to_owned(),
            list: Vec::from(vector),
        }
    }

    /// Parse a feature vector from a tab-separated line.
    ///
    /// Format: `name\tval1,val2,...`.
    /// Empty lines and lines starting with `#` are accepted and return an empty
    /// vector (callers such as `load_feature_vectors` skip them). Any other
    /// line with an invalid number of columns or non-numeric values returns an
    /// error.
    ///
    /// ```ignore
    /// # use necom::libs::clust::feature::FeatureVector;
    /// let line = "Es_coli_005008_GCF_013426115_1\t1,5,2,7,6,6".to_string();
    /// let entry = FeatureVector::parse(&line).unwrap();
    /// # assert_eq!(*entry.name(), "Es_coli_005008_GCF_013426115_1");
    /// # assert_eq!(*entry.list().get(1).unwrap(), 5f32);
    /// ```
    pub fn parse(line: &str) -> anyhow::Result<FeatureVector> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(Self::new());
        }
        let fields: Vec<&str> = trimmed.split('\t').collect();
        if fields.len() != 2 {
            anyhow::bail!("expected two tab-separated columns, got {}", fields.len());
        }
        let name = fields[0].to_string();
        let parts: Vec<&str> = fields[1].split(',').collect();
        let list: Vec<f32> = parts
            .iter()
            .map(|e| {
                e.parse::<f32>()
                    .map_err(|e| anyhow!("invalid float value: {}", e))
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(Self::from(&name, &list))
    }
}

impl std::fmt::Display for FeatureVector {
    /// To string
    ///
    /// ```ignore
    /// # use necom::libs::clust::feature::FeatureVector;
    /// let name = "Es_coli_005008_GCF_013426115_1".to_string();
    /// let list : Vec<f32> = vec![1.0,5.0,2.0,7.0,6.0,6.0];
    /// let entry = FeatureVector::from(&name, &list);
    /// assert_eq!(entry.to_string(), "Es_coli_005008_GCF_013426115_1\t1,5,2,7,6,6\n");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}\t", self.name())?;
        for (i, e) in self.list.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", e)?;
        }
        writeln!(f)
    }
}

/// Load feature vectors from a file, optionally binarizing values to 0.0/1.0.
pub fn load_feature_vectors(
    infile: &str,
    is_bin: bool,
) -> anyhow::Result<Vec<FeatureVector>> {
    let mut entries = vec![];
    let reader = crate::libs::io::reader(infile)?;
    for line in reader.lines() {
        let line = line?;
        let mut entry = FeatureVector::parse(&line)?;
        if entry.name().is_empty() {
            continue;
        }
        if is_bin {
            let bin_list = entry
                .list()
                .iter()
                .map(|e| if *e > 0.0 { 1.0 } else { 0.0 })
                .collect::<Vec<f32>>();
            entry = FeatureVector::from(entry.name(), &bin_list);
        }
        entries.push(entry);
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_valid() {
        let line = "A\t1.0,2.0,3.0";
        let fv = FeatureVector::parse(line).unwrap();
        assert_eq!(fv.name(), "A");
        assert_eq!(fv.list(), &vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_parse_empty_and_comment_lines() {
        let empty = FeatureVector::parse("").unwrap();
        assert!(empty.name().is_empty());
        assert!(empty.list().is_empty());

        let spaces = FeatureVector::parse("   ").unwrap();
        assert!(spaces.name().is_empty());

        let comment = FeatureVector::parse("# comment").unwrap();
        assert!(comment.name().is_empty());
    }

    #[test]
    fn test_parse_wrong_column_count() {
        assert!(FeatureVector::parse("A").is_err());
        assert!(FeatureVector::parse("A\t1.0\t2.0").is_err());
    }

    #[test]
    fn test_parse_invalid_float() {
        assert!(FeatureVector::parse("A\t1.0,foo,3.0").is_err());
    }

    #[test]
    fn test_display() {
        let fv = FeatureVector::from("A", &[1.0, 2.0, 3.0]);
        assert_eq!(fv.to_string(), "A\t1,2,3\n");
    }

    #[test]
    fn test_load_feature_vectors() -> anyhow::Result<()> {
        let temp = tempfile::TempDir::new()?;
        let path = temp.path().join("features.tsv");
        let mut file = std::fs::File::create(&path)?;
        writeln!(file, "A\t1.0,2.0")?;
        writeln!(file, "# comment")?;
        writeln!(file, "")?;
        writeln!(file, "B\t3.0,4.0")?;

        let entries = load_feature_vectors(path.to_str().unwrap(), false)?;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name(), "A");
        assert_eq!(entries[1].name(), "B");
        Ok(())
    }

    #[test]
    fn test_load_feature_vectors_malformed() -> anyhow::Result<()> {
        let temp = tempfile::TempDir::new()?;
        let path = temp.path().join("features.tsv");
        let mut file = std::fs::File::create(&path)?;
        writeln!(file, "A\t1.0,2.0")?;
        writeln!(file, "B\tbad")?;

        let result = load_feature_vectors(path.to_str().unwrap(), false);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_load_feature_vectors_binarize() -> anyhow::Result<()> {
        let temp = tempfile::TempDir::new()?;
        let path = temp.path().join("features.tsv");
        let mut file = std::fs::File::create(&path)?;
        writeln!(file, "A\t5.0,0.0,-3.0,2.5")?;
        writeln!(file, "B\t0.0,-0.1,0.1,100.0")?;

        let entries = load_feature_vectors(path.to_str().unwrap(), true)?;
        assert_eq!(entries.len(), 2);

        // All values must be 0.0 or 1.0 after binarization.
        for entry in &entries {
            for v in entry.list() {
                assert!(
                    *v == 0.0 || *v == 1.0,
                    "binarized value should be 0.0 or 1.0, got {}",
                    v
                );
            }
        }

        // Verify specific binarization outcomes: >0.0 -> 1.0, else 0.0.
        assert_eq!(entries[0].name(), "A");
        assert_eq!(entries[0].list(), &vec![1.0, 0.0, 0.0, 1.0]);

        assert_eq!(entries[1].name(), "B");
        assert_eq!(entries[1].list(), &vec![0.0, 0.0, 1.0, 1.0]);

        Ok(())
    }
}
