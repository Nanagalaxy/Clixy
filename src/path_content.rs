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
}

#[derive(Debug)]
pub enum IgnoreFlag {
    Files,
    Directories,
    None,
}

impl Default for IgnoreFlag {
    fn default() -> Self {
        IgnoreFlag::None
    }
}

impl PathContent {
    pub fn new() -> Self {
        PathContent {
            entries: 0,
            size: 0,
            list_of_dirs: vec![],
            list_of_files: vec![],
        }
    }

    pub fn index_entries(&mut self, path: &Path, into: bool, ignore: IgnoreFlag) -> Result<()> {
        let pb = progress_bar_helper::create_spinner();

        pb.set_message(format!("Indexing entries: {}", self.entries));

        let mut list_to_explore = if into {
            // The source path will be copied directly into the destination path
            vec![path.to_path_buf()]
        } else {
            // The contents of the source path will be copied into the destination path
            path.read_dir()?
                .par_bridge()
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    Some(path)
                })
                .collect()
        };

        while let Some(item) = list_to_explore.pop() {
            if item.is_dir() {
                match ignore {
                    IgnoreFlag::Directories => {
                        // Do not index directories
                        // Don't call continue here because we need to explore the directory content
                    }
                    _ => {
                        self.list_of_dirs.push(item.clone());
                        self.increment_entries(&pb);
                    }
                }

                match read_dir(item) {
                    Ok(entries) => {
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
                    }
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            "Error reading directory content",
                        ));
                    }
                }
            } else if item.is_file() {
                match ignore {
                    IgnoreFlag::Files => {
                        continue;
                    }
                    _ => {
                        // Let's index the file
                        // Push isn't done here because we need to check the metadata before indexing the file
                    }
                }

                match item.metadata() {
                    Ok(metadata) => {
                        self.size += metadata.len();
                    }
                    Err(_) => {
                        return Err(Error::new(ErrorKind::Other, "Error reading file metadata"));
                    }
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
