use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Open a buffered reader for a file, or stdin when `infile` is `"stdin"`.
pub fn reader<P: AsRef<Path>>(infile: P) -> anyhow::Result<Box<dyn BufRead>> {
    let infile = infile.as_ref();
    if infile.as_os_str() == "stdin" {
        Ok(Box::new(BufReader::new(io::stdin())))
    } else {
        Ok(Box::new(BufReader::new(File::open(infile)?)))
    }
}

/// Open a writer for a file, or stdout when `outfile` is `"stdout"`.
pub fn writer<P: AsRef<Path>>(outfile: P) -> anyhow::Result<Box<dyn Write>> {
    let outfile = outfile.as_ref();
    if outfile.as_os_str() == "stdout" {
        Ok(Box::new(io::stdout()))
    } else {
        Ok(Box::new(File::create(outfile)?))
    }
}

/// Read lines from a file or stdin, propagating IO errors (e.g. invalid UTF-8)
/// instead of silently truncating the stream.
pub fn read_lines<P: AsRef<Path>>(
    path: P,
) -> anyhow::Result<impl Iterator<Item = anyhow::Result<String>>> {
    let reader = reader(path)?;
    Ok(reader
        .lines()
        .map(|r| r.map_err(|e| anyhow::anyhow!("failed to read line: {}", e))))
}

/// Read whitespace-delimited names from a file or stdin.
///
/// Empty lines and lines whose first non-whitespace character is `#` are ignored.
/// IO errors (e.g. invalid UTF-8 bytes) are propagated rather than silently
/// truncating the stream.
pub fn read_names<C: FromIterator<String>>(file: &str) -> anyhow::Result<C> {
    let reader = reader(file)?;
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<_, _>>()
        .map_err(|e| anyhow::anyhow!("failed to read line from {}: {}", file, e))?;
    let names: C = lines
        .iter()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .flat_map(|line| {
            line.split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    Ok(names)
}

/// Read a replacement TSV file where the first column is the key and remaining
/// columns are replacement values.
///
/// Duplicate keys keep the first occurrence and warn. Lines with fewer than
/// two columns are skipped with a warning.
pub fn read_replace_tsv(file: &str) -> anyhow::Result<BTreeMap<String, Vec<String>>> {
    let mut map = BTreeMap::new();
    for line in read_lines(file)? {
        let line = line?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            log::warn!("skipping malformed line in replace file: {}", line);
            continue;
        }
        let name = parts[0].to_string();
        let replaces: Vec<String> =
            parts.iter().skip(1).map(|s| s.to_string()).collect();
        match map.entry(name) {
            std::collections::btree_map::Entry::Occupied(entry) => {
                log::warn!(
                    "duplicate replacement key '{}' in replace file, keeping first occurrence",
                    entry.key()
                );
            }
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(replaces);
            }
        }
    }
    Ok(map)
}

/// Return the current executable path as a UTF-8 string.
pub fn current_exe_string() -> anyhow::Result<String> {
    let exe = std::env::current_exe()?;
    exe.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("current executable path is not UTF-8"))
}

/// Convert a path to a UTF-8 string slice.
pub fn path_to_str(path: &Path) -> anyhow::Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow::anyhow!("path is not UTF-8: {}", path.display()))
}

/// Resolve a path to an absolute, normalized path.
///
/// If the path is relative, it is resolved against the current working directory.
/// Components such as `.` and `..` are collapsed.
pub fn absolute_path<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let path = path.as_ref();
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    Ok(clean_path(&absolute))
}

fn clean_path(path: &Path) -> PathBuf {
    let mut cleaned = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                cleaned.push(component.as_os_str());
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if cleaned.as_os_str().is_empty() {
                    cleaned.push("..");
                } else if cleaned.components().count() == 1
                    && matches!(
                        cleaned.components().next(),
                        Some(
                            std::path::Component::Prefix(_)
                                | std::path::Component::RootDir
                        )
                    )
                {
                    // Already at filesystem root or prefix; ignore `..`.
                } else if !cleaned.pop() {
                    cleaned.push("..");
                }
            }
            std::path::Component::Normal(name) => {
                cleaned.push(name);
            }
        }
    }
    if cleaned.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::{read_names, read_replace_tsv};
    use std::io::Write;

    #[test]
    fn test_read_names_skips_empty_and_comments() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "A").unwrap();
        writeln!(tmp).unwrap();
        writeln!(tmp, "# comment").unwrap();
        writeln!(tmp, "  # indented comment").unwrap();
        writeln!(tmp, "B C").unwrap();

        let names: Vec<String> = read_names(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(names, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_read_replace_tsv_basic() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "A\tX").unwrap();
        writeln!(tmp, "B\tY\tZ").unwrap();
        tmp.flush().unwrap();

        let map = read_replace_tsv(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(map.get("A").unwrap(), &vec!["X".to_string()]);
        assert_eq!(
            map.get("B").unwrap(),
            &vec!["Y".to_string(), "Z".to_string()]
        );
    }

    #[test]
    fn test_read_replace_tsv_duplicate_key_keeps_first() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "A\tfirst").unwrap();
        writeln!(tmp, "A\tsecond").unwrap();
        tmp.flush().unwrap();

        let map = read_replace_tsv(tmp.path().to_str().unwrap()).unwrap();
        // First occurrence wins; "second" is discarded with a warning.
        assert_eq!(map.get("A").unwrap(), &vec!["first".to_string()]);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_read_replace_tsv_skips_short_lines() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "A\tX").unwrap();
        writeln!(tmp, "lonely").unwrap();
        writeln!(tmp, "B\tY").unwrap();
        tmp.flush().unwrap();

        let map = read_replace_tsv(tmp.path().to_str().unwrap()).unwrap();
        // Line with <2 columns is skipped.
        assert!(!map.contains_key("lonely"));
        assert_eq!(map.len(), 2);
    }
}
