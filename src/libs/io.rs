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

/// Read lines from a file or stdin, skipping lines that fail to read.
pub fn read_lines<P: AsRef<Path>>(path: P) -> anyhow::Result<impl Iterator<Item = String>> {
    let reader = reader(path)?;
    Ok(reader.lines().map_while(Result::ok))
}

/// Read whitespace-delimited names from a file or stdin.
pub fn read_names<C: FromIterator<String>>(file: &str) -> anyhow::Result<C> {
    let reader = reader(file)?;
    let names: C = reader
        .lines()
        .map_while(Result::ok)
        .flat_map(|line| {
            line.split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    Ok(names)
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
                        Some(std::path::Component::Prefix(_) | std::path::Component::RootDir)
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
