use super::BaseCmdOpt;
use crate::path_content::PathContent;
use crate::progress_bar_helper;
use crate::utils::{add_error, confirm_continue};
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
pub struct RemoveCommand {
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

pub fn execute_remove(cmd: RemoveCommand) {
    let RemoveCommand {
        source,
        base: BaseCmdOpt { workers },
        only_files,
        yes,
        content_only,
    } = cmd;

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
                println!("Aborting remove");
                return;
            }
        }
    }

    let source_path = Path::new(&source);

    let mut path_content = PathContent::new();

    if let Err(_) = path_content.index_entries(source_path, content_only) {
        eprintln!("Error indexing source path, aborting remove");
        return;
    }

    if path_content.entries == 0 {
        println!("Source path is empty, nothing to remove");
        return;
    }

    if !yes {
        println!(
            "Removing {} files and {} directories from {} ({} entries)",
            path_content.list_of_files.len(),
            path_content.list_of_dirs.len(),
            source_path.display(),
            path_content.entries
        );

        if !confirm_continue() {
            println!("Aborting remove");
            return;
        }
    }

    let list_of_errors = Arc::new(Mutex::new(vec![]));

    if !path_content.list_of_files.is_empty() {
        remove_files(&path_content, &list_of_errors);
    } else {
        println!("No files to remove");
    }

    if !only_files && !path_content.list_of_dirs.is_empty() {
        remove_dirs(&path_content, &list_of_errors, source_path);
    } else {
        println!("No directories to remove or directories removal skipped");
    }

    let list_of_errors = match Arc::try_unwrap(list_of_errors) {
        Ok(list_of_errors) => list_of_errors.into_inner().unwrap_or(vec![]),
        Err(_) => {
            eprintln!("Error getting list of errors, somethings went wrong");
            return;
        }
    };

    if list_of_errors.is_empty() {
        println!("Remove completed successfully");
    } else {
        eprintln!(
            "{} error(s) occurred during the remove :",
            list_of_errors.len()
        );
        for error in list_of_errors {
            eprintln!("- {}", error);
        }
    }
}

fn remove_files(path_content: &PathContent, list_of_errors: &Arc<Mutex<Vec<String>>>) {
    let pb = progress_bar_helper::create_progress(path_content.list_of_files.len() as u64);

    pb.set_message("Removing files");

    path_content.list_of_files.par_iter().for_each(|item| {
        if let Err(_) = remove_file(item) {
            add_error(list_of_errors, format!("Error removing file {:?}", item));
            return;
        }

        pb.inc(1);
    });

    pb.finish_with_message("Files removed");
}

fn remove_dirs(
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

        if let Err(_) = remove_dir(item) {
            add_error(
                list_of_errors,
                format!("Error removing directory {:?}", item),
            );
            return;
        }

        pb.inc(1);
    });

    pb.finish_with_message("Directories removed");
}
