use crate::progress_bar_helper;
use indicatif::ProgressBar;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::fs::read_dir;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct PathContent {
    /// The number of entries in the path
    pub entries: u64,

    /// The size of the path in bytes
    pub size: u64,

    /// A list of directories in the path
    pub list_of_dirs: Vec<PathBuf>,

    /// A list of files in the path
    pub list_of_files: Vec<PathBuf>,

    // Indicates if the index has been created or not
    indexed: bool,
}

#[derive(Debug, Default)]
pub enum IgnoreFlag {
    Files,
    Directories,
    #[default]
    None,
}

impl PathContent {
    pub fn new() -> Self {
        PathContent {
            entries: 0,
            size: 0,
            list_of_dirs: vec![],
            list_of_files: vec![],
            indexed: false,
        }
    }

    pub fn index_entries(&mut self, path: &Path, into: bool, ignore: &IgnoreFlag) -> Result<()> {
        if self.indexed {
            return Err(Error::new(
                ErrorKind::Other,
                "The path content has already been indexed",
            ));
        }

        self.indexed = true;

        let pb = progress_bar_helper::create_spinner();

        pb.set_message(format!("Indexing entries: {}", self.entries));

        let mut list_to_explore = if into {
            // The source path will be copied directly into the destination path
            vec![path.to_path_buf()]
        } else if path.is_dir() {
            // The contents of the source path will be copied into the destination path
            path.read_dir()?
                .par_bridge()
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    Some(path)
                })
                .collect()
        } else {
            // For a file, we only need to copy the file itself
            vec![path.to_path_buf()]
        };

        while let Some(item) = list_to_explore.pop() {
            if item.is_dir() {
                if let IgnoreFlag::Directories = ignore {
                    // Do not index directories
                    // Don't call continue here because we need to explore the directory content
                } else {
                    self.list_of_dirs.push(item.clone());
                    self.increment_entries(&pb);
                }

                if let Ok(entries) = read_dir(item) {
                    for entry in entries {
                        match entry {
                            Ok(entry) => {
                                list_to_explore.push(entry.path());
                            }
                            Err(_) => {
                                return Err(Error::new(
                                    ErrorKind::Other,
                                    "Error reading directory content",
                                ));
                            }
                        }
                    }
                } else {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "Error reading directory content",
                    ));
                }
            } else if item.is_file() {
                if let IgnoreFlag::Files = ignore {
                    // Do not index files
                    continue;
                }

                // Let's index the file

                if let Ok(metadata) = item.metadata() {
                    self.size += metadata.len();
                } else {
                    return Err(Error::new(ErrorKind::Other, "Error reading file metadata"));
                }

                self.list_of_files.push(item);
                self.increment_entries(&pb);
            } else {
                return Err(Error::new(ErrorKind::Other, "Error processing source path"));
            }
        }

        pb.finish_with_message(format!("Indexed entries: {}", self.entries));

        Ok(())
    }

    fn increment_entries(&mut self, pb: &ProgressBar) {
        self.entries += 1;
        pb.set_message(format!("Indexing entries: {}", self.entries));
    }
}

#[test]
fn test_index_entries_file() {
    let mut path_content = PathContent::new();

    path_content
        .index_entries(Path::new("Cargo.toml"), true, &IgnoreFlag::None)
        .unwrap();

    assert_eq!(path_content.entries, 1);
    assert_eq!(path_content.list_of_files.len(), 1);
    assert_eq!(path_content.list_of_dirs.len(), 0);
}

#[test]
fn test_index_entries_ignore_files() {
    let mut path_content = PathContent::new();

    path_content
        .index_entries(Path::new("Cargo.toml"), true, &IgnoreFlag::Files)
        .unwrap();

    assert_eq!(path_content.entries, 0);
    assert_eq!(path_content.list_of_files.len(), 0);
    assert_eq!(path_content.list_of_dirs.len(), 0);
}

#[test]
fn test_index_entries_ignore_dirs() {
    let mut path_content = PathContent::new();

    path_content
        .index_entries(Path::new("Cargo.toml"), true, &IgnoreFlag::Directories)
        .unwrap();

    assert_eq!(path_content.entries, 1);
    assert_eq!(path_content.list_of_files.len(), 1);
    assert_eq!(path_content.list_of_dirs.len(), 0);
}
