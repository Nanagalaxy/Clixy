use crate::utils::{check_permissions, confirm_continue, round_bytes_size};
use clap::{builder, Args};
use fs_extra::dir::{get_dir_content, DirContent};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fs::remove_dir;
use std::io::{Error, ErrorKind, Result};
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
        help = "The source path to copy"
    )]
    source: String,

    #[arg(
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Only remove files, not directories"
    )]
    only_files: bool,

    #[arg(
        short,
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Confirm the remove operation before proceeding"
    )]
    yes: bool,
}

pub fn execute_remove(cmd: RemoveCommand) {
    let RemoveCommand {
        source,
        only_files,
        yes,
    } = cmd;

    let source_path = Path::new(&source);

    let remove_result = do_remove(source_path, only_files, yes);

    if remove_result {
        println!("Successfully removed {}", source);
    } else {
        eprintln!("Failed to remove {}", source);
    }
}

fn do_remove(source_path: &Path, only_files: bool, prompt_confirm: bool) -> bool {
    let m = MultiProgress::new();

    let dir_content = match get_dir_content(source_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Error reading contents of source folder, the path may not exist");
            return false;
        }
    };

    if !prompt_confirm {
        println!(
            "Removing {} files and {} directories from {} ({})",
            dir_content.files.len(),
            dir_content.directories.len(),
            source_path.display(),
            round_bytes_size(dir_content.dir_size)
        );

        if !confirm_continue() {
            println!("Aborting remove");
            return false;
        }
    }

    let pb_check = m.add(ProgressBar::new(dir_content.files.len() as u64));
    pb_check.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    // Start the progress bar
    pb_check.set_message("Checking source files");

    // Wrap the progress bar to handle parallel iterations
    let pb_check = Arc::new(pb_check);

    // Start a thread to handle the progress bar
    let pb_check_clone = Arc::clone(&pb_check);
    let ticker = thread::spawn(move || {
        while !pb_check_clone.is_finished() {
            pb_check_clone.tick();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Set a flag if any file is not accessible
    let check_error = Arc::new(Mutex::new(false));

    dir_content.files.par_iter().for_each(|item| {
        let pb_check = Arc::clone(&pb_check);

        match check_permissions(Path::new(item), true) {
            Ok(permissions) => {
                if !permissions.read {
                    eprintln!("Source file {:?} not readable", item);
                    *check_error.lock().unwrap() = true;
                } else if !permissions.write {
                    eprintln!("Source file {:?} not writable", item);
                    *check_error.lock().unwrap() = true;
                }
            }
            Err(_) => {
                eprintln!("Source file {:?} not accessible", item);
                *check_error.lock().unwrap() = true;
            }
        }

        pb_check.inc(1);
    });

    if *check_error.lock().unwrap() {
        pb_check.finish_with_message("Error checking source files, aborting remove");
        return false;
    } else {
        pb_check.finish_with_message("Source files checked successfully");
    }

    // Wait for the ticker thread to finish
    ticker.join().unwrap();

    if let Err(_) = remove_files(&dir_content) {
        return false;
    }

    if !only_files {
        if let Err(_) = remove_directories(&dir_content) {
            return false;
        }
    }

    true
}

fn remove_files(dir_content: &DirContent) -> Result<()> {
    let m = MultiProgress::new();

    let pb_remove = m.add(ProgressBar::new(dir_content.files.len() as u64));
    pb_remove.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    pb_remove.set_message("Removing source files");
    let pb_remove = Arc::new(pb_remove);

    let pb_remove_clone = Arc::clone(&pb_remove);
    let ticker = thread::spawn(move || {
        while !pb_remove_clone.is_finished() {
            pb_remove_clone.tick();
            thread::sleep(Duration::from_millis(100));
        }
    });

    let remove_error = Arc::new(Mutex::new(false));

    dir_content.files.par_iter().for_each(|item| {
        let pb_remove = Arc::clone(&pb_remove);

        if let Err(_) = remove_file(item) {
            eprintln!("Error removing file {:?}", item);
            *remove_error.lock().unwrap() = true;
        }

        pb_remove.inc(1);
    });

    if *remove_error.lock().unwrap() {
        pb_remove.finish_with_message("Error removing source files");
        return Err(Error::new(ErrorKind::Other, "Error removing source files"));
    } else {
        pb_remove.finish_with_message("Source files removed successfully");
    }

    ticker.join().unwrap();

    Ok(())
}

fn remove_directories(dir_content: &DirContent) -> Result<()> {
    let m = MultiProgress::new();

    let pb_remove = m.add(ProgressBar::new(dir_content.directories.len() as u64));
    pb_remove.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    pb_remove.set_message("Removing source directories");
    let pb_remove = Arc::new(pb_remove);

    let pb_remove_clone = Arc::clone(&pb_remove);
    let ticker = thread::spawn(move || {
        while !pb_remove_clone.is_finished() {
            pb_remove_clone.tick();
            thread::sleep(Duration::from_millis(100));
        }
    });

    let remove_error = Arc::new(Mutex::new(false));

    // Remove directories in reverse order to ensure the directory is empty
    dir_content.directories.iter().rev().for_each(|item| {
        let pb_remove = Arc::clone(&pb_remove);

        // Ensure the directory is empty before removing
        if let Err(_) = remove_dir(item) {
            eprintln!("Error removing directory {:?}", item);
            *remove_error.lock().unwrap() = true;
        }

        pb_remove.inc(1);
    });

    if *remove_error.lock().unwrap() {
        pb_remove.finish_with_message("Error removing source directories");
        return Err(Error::new(
            ErrorKind::Other,
            "Error removing source directories",
        ));
    } else {
        pb_remove.finish_with_message("Source directories removed successfully");
    }

    ticker.join().unwrap();

    Ok(())
}
