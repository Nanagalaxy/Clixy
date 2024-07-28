use clap::{builder, crate_authors, crate_description, crate_version, Args, Parser, Subcommand};
use fs_extra::dir::{get_dir_content, CopyOptions, DirContent};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::fs::{copy, create_dir_all, read_dir, File};
use std::io::{Error, ErrorKind, Read, Result};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Function for calculating the SHA-256 hash of a file
fn calculate_hash(file_path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);

    Ok(hasher.finalize().to_vec())
}

// Function to check the accessibility of a file or folder
fn check_accessibility(path: &Path) -> Result<()> {
    if path.is_file() {
        File::open(path).map(|_| ())
    } else if path.is_dir() {
        read_dir(path).map(|_| ())
    } else {
        Err(Error::new(ErrorKind::NotFound, "The path doesn't exist"))
    }
}

fn copy_directories(
    source_path: &Path,
    destination_path: &Path,
    dir_content: &DirContent,
) -> Result<()> {
    let m = MultiProgress::new();

    let pb_copy = m.add(ProgressBar::new(dir_content.directories.len() as u64));
    pb_copy.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    pb_copy.set_message("Copying directories");
    let pb_copy = Arc::new(pb_copy);

    let pb_copy_clone = Arc::clone(&pb_copy);
    let ticker = thread::spawn(move || {
        while !pb_copy_clone.is_finished() {
            pb_copy_clone.tick();
            thread::sleep(Duration::from_millis(100));
        }
    });

    for dir in &dir_content.directories {
        let relative_path = match Path::new(dir).strip_prefix(source_path) {
            Ok(rel_path) => rel_path,
            Err(_) => {
                eprintln!("Impossible to determine relative path for {:?}", dir);
                return Err(Error::new(
                    ErrorKind::Other,
                    "Failed to determine relative path",
                ));
            }
        };

        let destination_dir = destination_path.join(relative_path);

        if let Err(e) = create_dir_all(&destination_dir) {
            eprintln!("Unable to create directory {:?}: {:?}", destination_dir, e);
            return Err(Error::new(ErrorKind::Other, "Failed to create directory"));
        }

        pb_copy.inc(1);
    }

    pb_copy.finish_with_message("Directories copied successfully");

    ticker.join().unwrap();

    Ok(())
}

fn copy_files(source_path: &Path, destination_path: &Path, dir_content: &DirContent) -> Result<()> {
    let m = MultiProgress::new();

    let pb_copy = m.add(ProgressBar::new(dir_content.files.len() as u64));
    pb_copy.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    pb_copy.set_message("Copying files");
    let pb_copy = Arc::new(pb_copy);

    let pb_copy_clone = Arc::clone(&pb_copy);
    let ticker = thread::spawn(move || {
        while !pb_copy_clone.is_finished() {
            pb_copy_clone.tick();
            thread::sleep(Duration::from_millis(100));
        }
    });

    dir_content.files.par_iter().for_each(|item| {
        let pb_copy = Arc::clone(&pb_copy);

        let relative_path = match Path::new(item).strip_prefix(source_path) {
            Ok(rel_path) => rel_path,
            Err(_) => {
                eprintln!("Impossible to determine relative path for {:?}", item);
                return;
            }
        };

        let destination_file = destination_path.join(relative_path);

        if let Err(e) = copy(item, &destination_file) {
            eprintln!(
                "Error copying file {:?} to {:?}: {:?}",
                item, destination_file, e
            );
            return;
        }

        pb_copy.inc(1);
    });

    pb_copy.finish_with_message("Files copied successfully");

    ticker.join().unwrap();

    Ok(())
}

#[derive(PartialEq)]
enum CopyTypesOptions {
    None,
    Replace,
    Complete,
    Update,
}

fn do_copy(
    source_path: &Path,
    destination_path: &Path,
    option: CopyTypesOptions,
    only_folders: bool,
) -> Result<DirContent> {
    let m = MultiProgress::new();

    // Retrieves the contents of the source folder
    let mut dir_content = match get_dir_content(source_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Error reading contents of source folder, the path may not exist");
            return Err(Error::new(ErrorKind::NotFound, "The path doesn't exist"));
        }
    };

    // Create a progress bar for the check of the source files
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

        if let Err(_) = check_accessibility(Path::new(item)) {
            eprintln!("Source file {:?} not accessible", item);
            *check_error.lock().unwrap() = true;
        }

        pb_check.inc(1);
    });

    if *check_error.lock().unwrap() {
        pb_check.finish_with_message("Error checking source files, aborting copy");
        return Err(Error::new(ErrorKind::Other, "Error checking source files"));
    } else {
        pb_check.finish_with_message("Source files checked successfully");
    }

    // Wait for the ticker thread to finish
    ticker.join().unwrap();

    // Checks that the destination folder is accessible
    if destination_path.exists() {
        if let Err(_) = check_accessibility(destination_path) {
            eprintln!("Destination folder not accessible, check the path or permissions");
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Destination folder not accessible",
            ));
        }

        // Checks that the destination folder is empty
        if let Ok(content) = read_dir(destination_path) {
            if option == CopyTypesOptions::None && content.count() > 0 {
                eprintln!("Destination folder is not empty, please provide an empty folder or use an option");
                return Err(Error::new(
                    ErrorKind::AlreadyExists,
                    "Destination folder is not empty",
                ));
            }
        }
    } else {
        // Creates destination folder if none exists
        if let Err(_) = create_dir_all(destination_path) {
            eprintln!("Unable to create destination folder, check the path or permissions");
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Unable to create destination folder",
            ));
        }
    }

    // Copy directories first
    if let Err(e) = copy_directories(source_path, destination_path, &dir_content) {
        return Err(e);
    }

    if only_folders {
        // If only_folders is set, return early with an empty DirContent
        return Ok(DirContent {
            dir_size: 0,
            directories: Vec::new(),
            files: Vec::new(),
        });
    }

    // Copy options
    let mut options = CopyOptions::new();
    options.copy_inside = true; // Copies the contents of the folder rather than the folder itself

    // Remove the files in the list that are already in the destination folder if the complete flag is set
    if option == CopyTypesOptions::Complete {
        dir_content.files.retain(|item| {
            let destination_file =
                destination_path.join(match Path::new(item).strip_prefix(source_path) {
                    Ok(rel_path) => rel_path,
                    Err(_) => {
                        eprintln!("Impossible to determine relative path for {:?}", item);
                        return false;
                    }
                });

            !destination_file.exists()
        });
    } else if option == CopyTypesOptions::Update {
        // Remove the files in the list that are already in the destination folder and are older than the source files
        dir_content.files.retain(|item| {
            let destination_file =
                destination_path.join(match Path::new(item).strip_prefix(source_path) {
                    Ok(rel_path) => rel_path,
                    Err(_) => {
                        eprintln!("Impossible to determine relative path for {:?}", item);
                        return false;
                    }
                });

            if !destination_file.exists() {
                return true;
            }

            let source_metadata = match File::open(item) {
                Ok(metadata) => metadata.metadata().unwrap(),
                Err(_) => {
                    eprintln!("Error reading source file metadata for {:?}", item);
                    return false;
                }
            };

            let destination_metadata = match File::open(&destination_file) {
                Ok(metadata) => metadata.metadata().unwrap(),
                Err(_) => {
                    eprintln!(
                        "Error reading destination file metadata for {:?}",
                        destination_file
                    );
                    return false;
                }
            };

            source_metadata.modified().unwrap() > destination_metadata.modified().unwrap()
        });
    }

    // Copy files next
    if let Err(e) = copy_files(source_path, destination_path, &dir_content) {
        return Err(e);
    }

    Ok(dir_content)
}

fn verify_copy(source_path: &Path, destination_dir_content: DirContent) -> bool {
    // Check if destination_dir_content is empty
    if destination_dir_content.files.is_empty() {
        eprintln!("No destination files to verify");
        return true;
    }

    let m = MultiProgress::new();

    // Create a progress bar for the verification of the source files
    let pb_verify = m.add(ProgressBar::new(destination_dir_content.files.len() as u64));
    pb_verify.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    // Start the progress bar
    pb_verify.set_message("Verifying destination files");

    // Wrap the progress bar to handle parallel iterations
    let pb_verify = Arc::new(pb_verify);

    // Start a thread to handle the progress bar
    let pb_verify_clone = Arc::clone(&pb_verify);
    let ticker = thread::spawn(move || {
        while !pb_verify_clone.is_finished() {
            pb_verify_clone.tick();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Set a flag if any file is not accessible
    let verify_error = Arc::new(Mutex::new(false));

    destination_dir_content.files.par_iter().for_each(|item| {
        let pb_verify = Arc::clone(&pb_verify);

        let source_file = source_path.join(match Path::new(item).strip_prefix(source_path) {
            Ok(rel_path) => rel_path,
            Err(_) => {
                eprintln!("Impossible to determine relative path for {:?}", item);
                return;
            }
        });

        let source_hash = match calculate_hash(&source_file) {
            Ok(hash) => hash,
            Err(_) => {
                eprintln!("Error calculating hash for source file {:?}", source_file);
                return;
            }
        };

        let destination_hash = match calculate_hash(Path::new(item)) {
            Ok(hash) => hash,
            Err(_) => {
                eprintln!("Error calculating hash for destination file {:?}", item);
                return;
            }
        };

        if source_hash != destination_hash {
            eprintln!("File {:?} is different from the source", item);
            *verify_error.lock().unwrap() = true;
        }

        pb_verify.inc(1);
    });

    if *verify_error.lock().unwrap() {
        pb_verify.finish_with_message("Error verifying destination files");
    } else {
        pb_verify.finish_with_message("Destination files verified successfully");
    }

    // Wait for the ticker thread to finish
    ticker.join().unwrap();

    let res = !*verify_error.lock().unwrap();
    res
}

#[derive(Parser)]
#[command(author = crate_authors!("\n"), version = crate_version!(), about = crate_description!())]
struct ArgsCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Clone)]
#[group(multiple = false)]
struct ArgsCopyPossiblesOptions {
    #[arg(short,
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Option for the copy. Replace the destination path and its contents if it exists. Cannot be used with any other option"
    )]
    replace: bool,

    #[arg(short,
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Option for the copy. Only copy files that are not already in the destination folder Cannot be used with any other option"
    )]
    complete: bool,

    #[arg(short,
        long,
        default_value = "false",
        value_parser = builder::BoolValueParser::new(),
        help = "Option for the copy. Update the destination files if they are older than the source files Cannot be used with any other option"
    )]
    update: bool,
}

#[derive(Subcommand, Clone)]
enum Commands {
    #[command(about = "Copy the source path to the destination path")]
    Copy {
        #[arg(
            short,
            long,
            required = true,
            value_parser = builder::NonEmptyStringValueParser::new(),
            help = "The source path to copy"
        )]
        source: String,

        #[arg(
            short,
            long,
            required = true,
            value_parser = builder::NonEmptyStringValueParser::new(),
            help = "The destination path to copy to. Will be created if it doesn't exist"
        )]
        destination: String,

        #[clap(flatten)]
        options: ArgsCopyPossiblesOptions,

        #[arg(
            long,
            default_value = "false",
            value_parser = builder::BoolValueParser::new(),
            help = "Also copy the target path if it is a folder"
        )]
        copy_target: bool,

        #[arg(
            long,
            default_value = "false",
            value_parser = builder::BoolValueParser::new(),
            help = "Only copy folders, not files"
        )]
        only_folders: bool,

        #[arg(
            long,
            default_value = "false",
            value_parser = builder::BoolValueParser::new(),
            help = "Disable the verification of destination files after copying"
        )]
        no_verify: bool,
    },
}

fn main() {
    let args = ArgsCli::parse();

    match args.command {
        Commands::Copy {
            source,
            mut destination,
            options:
                ArgsCopyPossiblesOptions {
                    replace,
                    complete,
                    update,
                },
            copy_target,
            only_folders,
            no_verify,
        } => {
            let option = match (replace, complete, update) {
                (true, false, false) => CopyTypesOptions::Replace,
                (false, true, false) => CopyTypesOptions::Complete,
                (false, false, true) => CopyTypesOptions::Update,
                _ => CopyTypesOptions::None,
            };

            let source_path = Path::new(&source);
            // let destination_path = Path::new(&destination);

            if copy_target && source_path.is_dir() {
                let source_path_filename = match source_path.file_name() {
                    Some(filename) => filename,
                    None => {
                        eprintln!("Error getting source path filename");
                        return;
                    }
                };

                let temp_destination_path = Path::new(&destination).join(source_path_filename);

                destination = match temp_destination_path.to_str() {
                    Some(path) => path.to_string(),
                    None => {
                        eprintln!("Error getting destination path");
                        return;
                    }
                };
            }

            let destination_path = Path::new(&destination);

            let copied_result = do_copy(&source_path, &destination_path, option, only_folders);

            if !no_verify && copied_result.is_ok() {
                let dir_content = match copied_result {
                    Ok(content) => content,
                    Err(_) => {
                        eprintln!("Error copying source files, cannot verify");
                        return;
                    }
                };

                verify_copy(&source_path, dir_content);
            }
        }
    }
}
