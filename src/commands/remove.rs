use super::BaseCmdOpt;
use crate::path_content::{IgnoreFlag, PathContent};
use crate::progress_bar_helper;
use crate::utils::{add_error, confirm_continue, round_bytes_size};
use clap::{builder, ArgAction, Args};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fs::remove_dir;
use std::{
    fs::remove_file,
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Args, Clone)]
pub struct Command {
    #[arg(
        short,
        long,
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The source path to remove from."
    )]
    source: String,

    #[clap(flatten)]
    base: BaseCmdOpt,

    #[arg(
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Remove only files, skipping all folders."
    )]
    only_files: bool,

    #[arg(
        short,
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Automatically confirms the removal operation without prompting for user confirmation."
    )]
    yes: bool,

    #[arg(
        long,
        default_value = "true",
        value_parser = builder::BoolValueParser::new(),
        action = ArgAction::SetFalse,
        help = "Remove only the content of the source path, not the source path itself."
    )]
    content_only: bool,
}

pub fn execute(cmd: Command) {
    let Command {
        source,
        base: BaseCmdOpt { workers },
        only_files,
        yes,
        content_only,
    } = cmd;

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
            println!("Aborting remove");
            return;
        }
    }

    let source_path = Path::new(&source);

    let mut path_content = PathContent::new();

    let ignore_flag = if only_files {
        IgnoreFlag::Directories
    } else {
        IgnoreFlag::default()
    };

    if path_content
        .index_entries(source_path, content_only, &ignore_flag)
        .is_err()
    {
        eprintln!("Error indexing source path, aborting remove");
        return;
    }

    if path_content.entries == 0 {
        println!("Source path is empty, nothing to remove");
        return;
    }

    if !yes {
        println!(
            "Removing {} files and {} directories from {} ({} entries, {})",
            path_content.list_of_files.len(),
            path_content.list_of_dirs.len(),
            source_path.display(),
            path_content.entries,
            round_bytes_size(path_content.size)
        );

        if !confirm_continue() {
            println!("Aborting remove");
            return;
        }
    }

    let list_of_errors = Arc::new(Mutex::new(vec![]));

    let files_ok;

    if path_content.list_of_files.is_empty() {
        files_ok = true;
        println!("No files to remove");
    } else {
        files_ok = remove_files(&path_content, &list_of_errors);
    }

    if files_ok && !path_content.list_of_dirs.is_empty() {
        remove_dirs(&path_content, &list_of_errors, source_path);
    } else {
        println!("No directories to remove or directories removal skipped");
    }

    let list_of_errors = if let Ok(list_of_errors) = Arc::try_unwrap(list_of_errors) {
        list_of_errors.into_inner().unwrap_or(vec![])
    } else {
        eprintln!("Error getting list of errors, somethings went wrong");
        return;
    };

    if list_of_errors.is_empty() {
        println!(
            "Removed {} files and {} directories from {} ({} entries, {})",
            path_content.list_of_files.len(),
            path_content.list_of_dirs.len(),
            source_path.display(),
            path_content.entries,
            round_bytes_size(path_content.size)
        );
    } else {
        eprintln!(
            "{} error(s) occurred during the remove :",
            list_of_errors.len()
        );
        for error in list_of_errors {
            eprintln!("- {error}");
        }
    }
}

/// Remove all files in the path content.
/// Returns true if all files were removed successfully, false otherwise.
/// Note: because of the parallel processing, a flag protected by a mutex is used to track the status.
/// At the end of the process, the mutex is unwrapped to get the final status. If an error with the mutex occurs,
/// the function returns false.
#[allow(clippy::module_name_repetitions)]
pub fn remove_files(path_content: &PathContent, list_of_errors: &Arc<Mutex<Vec<String>>>) -> bool {
    let pb = progress_bar_helper::create_progress(path_content.list_of_files.len() as u64);

    pb.set_message("Removing files");

    let is_ok = Mutex::new(true);

    path_content.list_of_files.par_iter().for_each(|item| {
        if remove_file(item).is_err() {
            add_error(list_of_errors, format!("Error removing file {item:?}"));
            if let Ok(mut is_ok) = is_ok.lock() {
                *is_ok = false;
            }
            return;
        }

        pb.inc(1);
    });

    pb.finish_with_message("Files removed");

    is_ok.into_inner().unwrap_or(false)
}

#[allow(clippy::module_name_repetitions)]
pub fn remove_dirs(
    path_content: &PathContent,
    list_of_errors: &Arc<Mutex<Vec<String>>>,
    source_path: &Path,
) {
    let pb = progress_bar_helper::create_progress(path_content.list_of_dirs.len() as u64);

    pb.set_message("Removing directories");

    path_content.list_of_dirs.par_iter().for_each(|item| {
        // Check if the source path is in the list of directories
        if item == source_path {
            // Wait util the source path is empty before removing it
            while let Ok(content) = source_path.read_dir() {
                if content.count() == 0 {
                    break;
                }

                thread::sleep(Duration::from_millis(100));
            }
        }

        if remove_dir(item).is_err() {
            add_error(list_of_errors, format!("Error removing directory {item:?}"));
            return;
        }

        pb.inc(1);
    });

    pb.finish_with_message("Directories removed");
}
