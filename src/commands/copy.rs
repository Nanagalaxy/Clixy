use super::BaseCmdOpt;
use crate::path_content::PathContent;
use crate::progress_bar_helper;
use crate::utils::{add_error, calculate_hash, confirm_continue, round_bytes_size};
use clap::{builder, Args};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fs::{copy, create_dir_all};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(PartialEq)]
pub enum CopyTypesOptions {
    None,
    Replace,
    Complete,
    Update,
}

#[derive(Args, Clone)]
#[group(multiple = false)]
struct ArgsCopyPossiblesOptions {
    #[arg(short,
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Replace the destination path and its contents if they already exist. Cannot be used with --complete or --update."
    )]
    replace: bool,

    #[arg(short,
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Only copy files that do not exist in the destination folder. Cannot be used with --replace or --update."
    )]
    complete: bool,

    #[arg(short,
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Update destination files only if they are older than the source files. Cannot be used with --replace or --complete."
    )]
    update: bool,
}

#[derive(Args, Clone)]
pub struct CopyCommand {
    #[arg(
        short,
        long,
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The source path to copy from."
    )]
    source: String,

    #[arg(
        short,
        long,
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The destination path to copy to. This will be created if it doesn't exist."
    )]
    destination: String,

    #[clap(flatten)]
    base: BaseCmdOpt,

    #[clap(flatten)]
    options: ArgsCopyPossiblesOptions,

    #[arg(
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "If the source is a folder, copy the folder itself to the destination"
    )]
    copy_target: bool,

    #[arg(
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Copy only folders, skipping all files."
    )]
    only_folders: bool,

    #[arg(
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Skip verification of files after copying them to the destination."
    )]
    no_verify: bool,
}

pub fn execute_copy(cmd: CopyCommand) {
    let CopyCommand {
        source,
        destination,
        base: BaseCmdOpt { workers },
        options:
            ArgsCopyPossiblesOptions {
                replace,
                complete,
                update,
            },
        copy_target,
        only_folders,
        no_verify,
    } = cmd;

    let option = match (replace, complete, update) {
        (true, false, false) => CopyTypesOptions::Replace,
        (false, true, false) => CopyTypesOptions::Complete,
        (false, false, true) => CopyTypesOptions::Update,
        _ => CopyTypesOptions::None,
    };

    match rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build_global()
    {
        Ok(_) => {}
        Err(_) => {
            eprintln!(
                "Error setting the number of threads for rayon, using default value {}",
                rayon::current_num_threads()
            );

            if !confirm_continue() {
                println!("Aborting copy");
                return;
            }
        }
    }

    let source_path = Path::new(&source);
    let destination_path = Path::new(&destination);

    let mut path_content = PathContent::new();

    if let Err(_) = path_content.index_entries(source_path, copy_target) {
        eprintln!("Error indexing source path, aborting copy");
        return;
    }

    if path_content.entries == 0 {
        println!("Source path is empty, nothing to remove");
        return;
    }

    if destination_path.exists() && option == CopyTypesOptions::None {
        let content = match destination_path.read_dir() {
            Ok(content) => content,
            Err(_) => {
                eprintln!(
                    "Error reading destination folder content, check the path or permissions"
                );
                return;
            }
        };

        if content.count() > 0 {
            eprintln!("Destination folder exists, please provide an empty folder or use an option");
            return;
        }
    } else {
        if let Err(_) = create_dir_all(destination_path) {
            eprintln!("Unable to create destination folder, check the path or permissions");
            return;
        }
    }

    let list_of_errors = Arc::new(Mutex::new(vec![]));

    if !path_content.list_of_dirs.is_empty() {
        copy_dirs(
            &path_content,
            source_path,
            destination_path,
            &list_of_errors,
            copy_target,
        );
    } else {
        println!("No directories to copy");
    }

    if !only_folders && !path_content.list_of_files.is_empty() {
        let copied_files = copy_files(
            &path_content,
            source_path,
            destination_path,
            &list_of_errors,
            copy_target,
            &option,
        );

        if !no_verify {
            verify_copy(copied_files, &list_of_errors);
        }
    } else {
        println!("No files to copy or files were skipped");
    }

    let list_of_errors = match Arc::try_unwrap(list_of_errors) {
        Ok(list_of_errors) => list_of_errors.into_inner().unwrap_or(vec![]),
        Err(_) => {
            eprintln!("Error getting list of errors, somethings went wrong");
            return;
        }
    };

    if list_of_errors.is_empty() {
        println!(
            "Copied {} files and {} directories from {} ({} entries, {})",
            path_content.list_of_files.len(),
            path_content.list_of_dirs.len(),
            source_path.display(),
            path_content.entries,
            round_bytes_size(path_content.size)
        );
    } else {
        eprintln!(
            "{} error(s) occurred during the copy :",
            list_of_errors.len()
        );
        for error in list_of_errors {
            eprintln!("- {}", error);
        }
    }
}

fn copy_dirs(
    path_content: &PathContent,
    source_path: &Path,
    destination_path: &Path,
    list_of_errors: &Arc<Mutex<Vec<String>>>,
    copy_target: bool,
) {
    let pb = progress_bar_helper::create_progress(path_content.list_of_dirs.len() as u64);

    pb.set_message("Copying directories");

    path_content.list_of_dirs.par_iter().for_each(|dir| {
        let relative_path: &Path;

        if copy_target {
            let parent_path = match source_path.parent() {
                Some(parent) => parent,
                None => {
                    add_error(
                        list_of_errors,
                        format!("Impossible to determine parent path for {:?}", source_path),
                    );
                    return;
                }
            };

            relative_path = match dir.strip_prefix(parent_path) {
                Ok(rel_path) => rel_path,
                Err(_) => {
                    add_error(
                        list_of_errors,
                        format!("Impossible to determine relative path for {:?}", dir),
                    );
                    return;
                }
            };
        } else {
            relative_path = match dir.strip_prefix(source_path) {
                Ok(rel_path) => rel_path,
                Err(_) => {
                    add_error(
                        list_of_errors,
                        format!("Impossible to determine relative path for {:?}", dir),
                    );
                    return;
                }
            };
        }

        let destination_dir = destination_path.join(relative_path);

        // Do the copy of the directories
        if let Err(e) = create_dir_all(&destination_dir) {
            add_error(
                list_of_errors,
                format!("Unable to create directory {:?}: {:?}", destination_dir, e),
            );
            return;
        }

        pb.inc(1);
    });

    pb.finish_with_message("Directories copied");
}

/// Returns a vector with the paths of the copied files (source and destination)
fn copy_files(
    path_content: &PathContent,
    source_path: &Path,
    destination_path: &Path,
    list_of_errors: &Arc<Mutex<Vec<String>>>,
    copy_target: bool,
    option: &CopyTypesOptions,
) -> Vec<(PathBuf, PathBuf)> {
    let pb = progress_bar_helper::create_progress(path_content.list_of_files.len() as u64);

    pb.set_message("Copying files");

    let copied_files: Arc<Mutex<Vec<(PathBuf, PathBuf)>>> = Arc::new(Mutex::new(Vec::new()));

    path_content.list_of_files.par_iter().for_each(|file| {
        let relative_path: &Path;

        if copy_target {
            let parent_path = match source_path.parent() {
                Some(parent) => parent,
                None => {
                    add_error(
                        list_of_errors,
                        format!("Impossible to determine parent path for {:?}", source_path),
                    );
                    return;
                }
            };

            relative_path = match file.strip_prefix(parent_path) {
                Ok(rel_path) => rel_path,
                Err(_) => {
                    add_error(
                        list_of_errors,
                        format!("Impossible to determine relative path for {:?}", file),
                    );
                    return;
                }
            };
        } else {
            relative_path = match file.strip_prefix(source_path) {
                Ok(rel_path) => rel_path,
                Err(_) => {
                    add_error(
                        list_of_errors,
                        format!("Impossible to determine relative path for {:?}", file),
                    );
                    return;
                }
            };
        }

        let destination_file = destination_path.join(relative_path);

        let need_copy = match option {
            CopyTypesOptions::None => true,
            CopyTypesOptions::Replace => true,
            CopyTypesOptions::Complete => !destination_file.exists(),
            CopyTypesOptions::Update => {
                if !destination_file.exists() {
                    true
                } else {
                    let source_metadata = match file.metadata() {
                        Ok(metadata) => metadata,
                        Err(_) => {
                            add_error(
                                list_of_errors,
                                format!("Error reading metadata for file {:?}", file),
                            );
                            return;
                        }
                    };

                    let destination_metadata = match destination_file.metadata() {
                        Ok(metadata) => metadata,
                        Err(_) => {
                            add_error(
                                list_of_errors,
                                format!("Error reading metadata for file {:?}", destination_file),
                            );
                            return;
                        }
                    };

                    let source_modified = match source_metadata.modified() {
                        Ok(modified) => modified,
                        Err(_) => {
                            add_error(
                                list_of_errors,
                                format!("Error reading modified time for file {:?}", file),
                            );
                            return;
                        }
                    };

                    let destination_modified = match destination_metadata.modified() {
                        Ok(modified) => modified,
                        Err(_) => {
                            add_error(
                                list_of_errors,
                                format!(
                                    "Error reading modified time for file {:?}",
                                    destination_file
                                ),
                            );
                            return;
                        }
                    };

                    source_modified > destination_modified
                }
            }
        };

        if need_copy {
            // Do the copy of the files
            if let Err(e) = copy(file, &destination_file) {
                add_error(
                    list_of_errors,
                    format!(
                        "Error copying file {:?} to {:?}: {:?}",
                        file, destination_file, e
                    ),
                );
                return;
            }

            match copied_files.lock() {
                Ok(mut copied_files) => copied_files.push((file.to_path_buf(), destination_file)),
                Err(_) => {
                    add_error(
                        list_of_errors,
                        format!(
                            "Error adding copied file {:?} to the list of copied files",
                            destination_file
                        ),
                    );
                }
            }
        }

        pb.inc(1);
    });

    pb.finish_with_message("Files copied");

    let copied_files = match Arc::try_unwrap(copied_files) {
        Ok(copied_files) => copied_files.into_inner().unwrap_or(Vec::new()),
        Err(_) => {
            add_error(list_of_errors, "Error getting copied files".to_string());
            vec![]
        }
    };

    copied_files
}

fn verify_copy(copied_files: Vec<(PathBuf, PathBuf)>, list_of_errors: &Arc<Mutex<Vec<String>>>) {
    let pb = progress_bar_helper::create_progress(copied_files.len() as u64);

    pb.set_message("Verifying files");

    copied_files
        .par_iter()
        .for_each(|(source_file, destination_file)| {
            let source_hash = match calculate_hash(source_file) {
                Ok(hash) => hash,
                Err(_) => {
                    add_error(
                        list_of_errors,
                        format!("Error calculating hash for source file {:?}", source_file),
                    );
                    return;
                }
            };

            let destination_hash = match calculate_hash(destination_file) {
                Ok(hash) => hash,
                Err(_) => {
                    add_error(
                        list_of_errors,
                        format!(
                            "Error calculating hash for destination file {:?}",
                            destination_file
                        ),
                    );
                    return;
                }
            };

            if source_hash != destination_hash {
                add_error(
                    list_of_errors,
                    format!(
                        "Hashes don't match for files {:?} and {:?}",
                        source_file, destination_file
                    ),
                );
            }

            pb.inc(1);
        });

    pb.finish_with_message("Files verified");
}
