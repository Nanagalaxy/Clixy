use super::{
    copy::{copy_dirs, copy_files, verify_copy, CopyTypesOptions},
    remove::{remove_dirs, remove_files},
    BaseCmdOpt,
};
use crate::{
    path_content::PathContent,
    utils::{confirm_continue, round_bytes_size},
};
use clap::{builder, Args};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Args, Clone)]
pub struct MoveCommand {
    #[arg(
        short,
        long,
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The source path to move from."
    )]
    pub source: String,

    #[arg(
        short,
        long,
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The destination path to move to. This will be created if it doesn't exist."
    )]
    destination: String,

    #[clap(flatten)]
    pub base: BaseCmdOpt,
}

pub fn execute_move(cmd: MoveCommand) {
    let MoveCommand {
        source,
        destination,
        base: BaseCmdOpt { workers },
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
                println!("Aborting move");
                return;
            }
        }
    }

    let source_path = Path::new(&source);
    let destination_path = Path::new(&destination);

    let mut path_content = PathContent::new();

    let into = false;

    if let Err(_) = path_content.index_entries(source_path, into) {
        eprintln!("Error indexing source path, aborting move");
        return;
    }

    if path_content.entries == 0 {
        println!("Source path is empty, nothing to move");
        return;
    }

    if destination_path.exists() {
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
            eprintln!("Destination folder exists and is not empty, aborting move");
            return;
        }
    } else {
        if let Err(_) = std::fs::create_dir_all(destination_path) {
            eprintln!("Error creating destination path, aborting move");
            return;
        }
    }

    let copy_list_of_errors = Arc::new(Mutex::new(vec![]));

    let mut dirs_ok = false;

    if !path_content.list_of_dirs.is_empty() {
        dirs_ok = copy_dirs(
            &path_content,
            source_path,
            destination_path,
            &copy_list_of_errors,
            into,
        );
    } else {
        println!("No directories to move");
    }

    if dirs_ok && !path_content.list_of_files.is_empty() {
        let option = CopyTypesOptions::None;

        let copied_files = copy_files(
            &path_content,
            source_path,
            destination_path,
            &copy_list_of_errors,
            into,
            &option,
        );

        // if !no_verify {
        verify_copy(copied_files, &copy_list_of_errors);
        // }
    } else {
        println!("No files to move");
    }

    let copy_list_of_errors = match Arc::try_unwrap(copy_list_of_errors) {
        Ok(list_of_errors) => list_of_errors.into_inner().unwrap_or(vec![]),
        Err(_) => {
            eprintln!("Error getting list of errors, somethings went wrong");
            return;
        }
    };

    if copy_list_of_errors.is_empty() {
        println!("First move phase completed (copying), starting second move phase (removing)");
    } else {
        eprintln!(
            "{} error(s) occurred during the copy (first move phase) :",
            copy_list_of_errors.len()
        );
        for error in copy_list_of_errors {
            eprintln!("- {}", error);
        }
    }

    let remove_list_of_errors = Arc::new(Mutex::new(vec![]));

    let mut files_ok = false;

    if !path_content.list_of_files.is_empty() {
        files_ok = remove_files(&path_content, &remove_list_of_errors);
    } else {
        println!("No files to remove");
    }

    // Add the source path to the list of directories to remove
    if !path_content
        .list_of_dirs
        .contains(&source_path.to_path_buf())
    {
        path_content.list_of_dirs.push(source_path.to_path_buf());
    }

    if files_ok && !path_content.list_of_dirs.is_empty() {
        remove_dirs(&path_content, &remove_list_of_errors, source_path);
    } else {
        println!("No directories to remove");
    }

    let remove_list_of_errors = match Arc::try_unwrap(remove_list_of_errors) {
        Ok(list_of_errors) => list_of_errors.into_inner().unwrap_or(vec![]),
        Err(_) => {
            eprintln!("Error getting list of errors, somethings went wrong");
            return;
        }
    };

    if remove_list_of_errors.is_empty() {
        println!(
            "Moved {} files and {} directories from {} to {} ({} entries, {})",
            path_content.list_of_files.len(),
            path_content.list_of_dirs.len(),
            source_path.display(),
            destination_path.display(),
            path_content.entries,
            round_bytes_size(path_content.size)
        );
    } else {
        eprintln!(
            "{} error(s) occurred during the remove (second move phase) :",
            remove_list_of_errors.len()
        );
        for error in remove_list_of_errors {
            eprintln!("- {}", error);
        }
    }
}
