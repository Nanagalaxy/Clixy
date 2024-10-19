use super::BaseCmdOpt;
use crate::path_content::{IgnoreFlag, PathContent};
use crate::progress_bar_helper;
use crate::utils::{add_error, calculate_hash, confirm_continue, round_bytes_size};
use clap::{builder, Args};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fs::{copy, create_dir_all};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(PartialEq)]
pub enum OptionsTypes {
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
pub struct Command {
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

pub fn execute(cmd: Command) {
    let Command {
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
        (true, false, false) => OptionsTypes::Replace,
        (false, true, false) => OptionsTypes::Complete,
        (false, false, true) => OptionsTypes::Update,
        _ => OptionsTypes::None,
    };

    if rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build_global()
        .is_err()
    {
        eprintln!(
            "Error setting the number of threads for rayon, using default value {}",
            rayon::current_num_threads()
        );

        if !confirm_continue() {
            println!("Aborting copy");
            return;
        }
    }

    let source_path = Path::new(&source);
    let destination_path = Path::new(&destination);

    let mut path_content = PathContent::new();

    let ignore_flag = if only_folders {
        IgnoreFlag::Files
    } else {
        IgnoreFlag::default()
    };

    if path_content
        .index_entries(source_path, copy_target, &ignore_flag)
        .is_err()
    {
        eprintln!("Error indexing source path, aborting copy");
        return;
    }

    if path_content.entries == 0 {
        println!("Source path is empty, nothing to copy");
        return;
    }

    if destination_path.exists() && option == OptionsTypes::None {
        let Ok(content) = destination_path.read_dir() else {
            eprintln!("Error reading destination folder content, check the path or permissions");
            return;
        };

        if content.count() > 0 {
            eprintln!("Destination folder exists and is not empty, please provide an empty folder or use an option");
            return;
        }
    } else if create_dir_all(destination_path).is_err() {
        eprintln!("Unable to create destination folder, check the path or permissions");
        return;
    } else {
        println!("Destination path created");
    }

    if let Ok(available_space) = fs4::available_space(destination_path) {
        if available_space < path_content.size {
            eprintln!(
                "Not enough space available in the destination folder ({} needed, {} available), aborting copy",
                round_bytes_size(path_content.size),
                round_bytes_size(available_space)
            );
            return;
        }
    } else {
        eprintln!("Error getting available space in the destination folder, aborting copy");
        return;
    }

    let list_of_errors = Arc::new(Mutex::new(vec![]));

    let dirs_ok;

    if path_content.list_of_dirs.is_empty() {
        dirs_ok = true;
        println!("No directories to copy");
    } else {
        dirs_ok = copy_dirs(
            &path_content,
            source_path,
            destination_path,
            &list_of_errors,
            copy_target,
        );
    }

    if dirs_ok && !path_content.list_of_files.is_empty() {
        let copied_files = copy_files(
            &path_content,
            source_path,
            destination_path,
            &list_of_errors,
            copy_target,
            &option,
        );

        if !no_verify {
            verify_copy(&copied_files, &list_of_errors);
        }
    } else {
        println!("No files to copy or files were skipped");
    }

    let list_of_errors = if let Ok(list_of_errors) = Arc::try_unwrap(list_of_errors) {
        list_of_errors.into_inner().unwrap_or(vec![])
    } else {
        eprintln!("Error getting list of errors, somethings went wrong");
        return;
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
            eprintln!("- {error}");
        }
    }
}

/// Copy directories from the source path to the destination path.
/// Returns true if the copy was successful, false otherwise.
/// Note: because of the parallel processing, a flag protected by a mutex is used to track the status.
/// At the end of the process, the mutex is unwrapped to get the final status. If an error with the mutex occurs,
/// the function returns false.
#[allow(clippy::module_name_repetitions)]
pub fn copy_dirs(
    path_content: &PathContent,
    source_path: &Path,
    destination_path: &Path,
    list_of_errors: &Arc<Mutex<Vec<String>>>,
    copy_target: bool,
) -> bool {
    let pb = progress_bar_helper::create_progress(path_content.list_of_dirs.len() as u64);

    pb.set_message("Copying directories");

    let is_ok = Mutex::new(true);

    path_content.list_of_dirs.par_iter().for_each(|dir| {
        let relative_path = if copy_target {
            let Some(parent_path) = source_path.parent() else {
                add_error(
                    list_of_errors,
                    format!("Impossible to determine parent path for {source_path:?}"),
                );
                if let Ok(mut is_ok) = is_ok.lock() {
                    *is_ok = false;
                }
                return;
            };

            let Ok(rel_path) = dir.strip_prefix(parent_path) else {
                add_error(
                    list_of_errors,
                    format!("Impossible to determine relative path for {dir:?}"),
                );
                if let Ok(mut is_ok) = is_ok.lock() {
                    *is_ok = false;
                }
                return;
            };

            rel_path
        } else {
            let Ok(rel_path) = dir.strip_prefix(source_path) else {
                add_error(
                    list_of_errors,
                    format!("Impossible to determine relative path for {dir:?}"),
                );
                if let Ok(mut is_ok) = is_ok.lock() {
                    *is_ok = false;
                }
                return;
            };

            rel_path
        };

        let destination_dir = destination_path.join(relative_path);

        // Do the copy of the director&ies
        if let Err(e) = create_dir_all(&destination_dir) {
            add_error(
                list_of_errors,
                format!("Unable to create directory {destination_dir:?}: {e:?}"),
            );
            if let Ok(mut is_ok) = is_ok.lock() {
                *is_ok = false;
            }
            return;
        }

        pb.inc(1);
    });

    pb.finish_with_message("Directories copied");

    is_ok.into_inner().unwrap_or(false)
}

/// Returns a vector with the paths of the copied files (source and destination)
#[allow(clippy::module_name_repetitions)]
pub fn copy_files(
    path_content: &PathContent,
    source_path: &Path,
    destination_path: &Path,
    list_of_errors: &Arc<Mutex<Vec<String>>>,
    copy_target: bool,
    option: &OptionsTypes,
) -> Vec<(PathBuf, PathBuf)> {
    let pb = progress_bar_helper::create_progress(path_content.list_of_files.len() as u64);

    pb.set_message("Copying files");

    let copied_files: Arc<Mutex<Vec<(PathBuf, PathBuf)>>> = Arc::new(Mutex::new(Vec::new()));

    path_content.list_of_files.par_iter().for_each(|file| {
        let relative_path = if copy_target {
            let Some(parent_path) = source_path.parent() else {
                add_error(
                    list_of_errors,
                    format!("Impossible to determine parent path for {source_path:?}"),
                );
                return;
            };

            let Ok(rel_path) = file.strip_prefix(parent_path) else {
                add_error(
                    list_of_errors,
                    format!("Impossible to determine relative path for {file:?}"),
                );
                return;
            };

            rel_path
        } else {
            let Ok(rel_path) = file.strip_prefix(source_path) else {
                add_error(
                    list_of_errors,
                    format!("Impossible to determine relative path for {file:?}"),
                );
                return;
            };

            rel_path
        };

        let destination_file = destination_path.join(relative_path);

        let need_copy = match option {
            OptionsTypes::None | OptionsTypes::Replace => true,
            OptionsTypes::Complete => !destination_file.exists(),
            OptionsTypes::Update => {
                if destination_file.exists() {
                    let Ok(source_metadata) = file.metadata() else {
                        add_error(
                            list_of_errors,
                            format!("Error reading metadata for file {file:?}"),
                        );
                        return;
                    };

                    let Ok(destination_metadata) = destination_file.metadata() else {
                        add_error(
                            list_of_errors,
                            format!("Error reading metadata for file {destination_file:?}"),
                        );
                        return;
                    };

                    let Ok(source_modified) = source_metadata.modified() else {
                        add_error(
                            list_of_errors,
                            format!("Error reading modified time for file {file:?}"),
                        );
                        return;
                    };

                    let Ok(destination_modified) = destination_metadata.modified() else {
                        add_error(
                            list_of_errors,
                            format!(
                                "Error reading modified time for file {destination_file:?}"
                            ),
                        );
                        return;
                    };

                    source_modified > destination_modified
                } else {
                    true
                }
            }
        };

        if need_copy {
            // Do the copy of the files
            if let Err(e) = copy(file, &destination_file) {
                add_error(
                    list_of_errors,
                    format!(
                        "Error copying file {file:?} to {destination_file:?}: {e:?}"
                    ),
                );
                return;
            }

            match copied_files.lock() {
                Ok(mut copied_files) => copied_files.push((file.clone(), destination_file)),
                Err(_) => {
                    add_error(
                        list_of_errors,
                        format!(
                            "Error adding copied file {destination_file:?} to the list of copied files"
                        ),
                    );
                }
            }
        }

        pb.inc(1);
    });

    pb.finish_with_message("Files copied");

    if let Ok(copied_files) = Arc::try_unwrap(copied_files) {
        copied_files.into_inner().unwrap_or(Vec::new())
    } else {
        add_error(list_of_errors, "Error getting copied files".to_string());
        vec![]
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn verify_copy(
    copied_files: &Vec<(PathBuf, PathBuf)>,
    list_of_errors: &Arc<Mutex<Vec<String>>>,
) {
    let pb = progress_bar_helper::create_progress(copied_files.len() as u64);

    pb.set_message("Verifying files");

    copied_files
        .par_iter()
        .for_each(|(source_file, destination_file)| {
            let Ok(source_hash) = calculate_hash(source_file) else {
                add_error(
                    list_of_errors,
                    format!("Error calculating hash for source file {source_file:?}"),
                );
                return;
            };

            let Ok(destination_hash) = calculate_hash(destination_file) else {
                add_error(
                    list_of_errors,
                    format!("Error calculating hash for destination file {destination_file:?}"),
                );
                return;
            };

            if source_hash != destination_hash {
                add_error(
                    list_of_errors,
                    format!(
                        "Hashes don't match for files {source_file:?} and {destination_file:?}"
                    ),
                );
            }

            pb.inc(1);
        });

    pb.finish_with_message("Files verified");
}
