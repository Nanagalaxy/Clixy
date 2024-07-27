use clap::{builder, crate_authors, crate_description, crate_version, Args, Parser, Subcommand};
use fs_extra::dir::{get_dir_content, CopyOptions};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::fs::{copy, create_dir, create_dir_all, read_dir, File};
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

#[derive(PartialEq)]
enum CopyPossiblesOptions {
    None,
    Replace,
    Complete,
    Update,
}

fn do_copy(source_path: &str, destination_path: &str, option: CopyPossiblesOptions) {
    let m = MultiProgress::new();

    // Retrieves the contents of the source folder
    let mut dir_content = match get_dir_content(source_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Error reading contents of source folder, the path may not exist");
            return;
        }
    };

    // Create a progress bar for the verification of the source files
    let pb_verify = m.add(ProgressBar::new(dir_content.files.len() as u64));
    pb_verify.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

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

    // Parallel verification
    dir_content.files.par_iter().for_each(|item| {
        let pb_verify = Arc::clone(&pb_verify);

        if let Err(_) = check_accessibility(Path::new(item)) {
            eprintln!("Source file {:?} not accessible", item);
            *verify_error.lock().unwrap() = true;
        }

        pb_verify.inc(1);
    });

    if *verify_error.lock().unwrap() {
        pb_verify.finish_with_message("Error verifying sources files, aborting copy");
        return;
    } else {
        pb_verify.finish_with_message("Files verified successfully");
    }

    // Wait for the ticker thread to finish
    ticker.join().unwrap();

    // Checks that the destination folder is accessible
    if Path::new(destination_path).exists() {
        if let Err(_) = check_accessibility(Path::new(destination_path)) {
            eprintln!("Destination folder not accessible, check the path or permissions");
            return;
        }

        // Checks that the destination folder is empty
        if let Ok(content) = read_dir(destination_path) {
            if option == CopyPossiblesOptions::None && content.count() > 0 {
                eprintln!("Destination folder is not empty, please provide an empty folder or use an option");
                return;
            }
        }
    } else {
        // Creates destination folder if none exists
        if let Err(_) = create_dir(destination_path) {
            eprintln!("Unable to create destination folder, check the path or permissions");
            return;
        }
    }

    // Copy options
    let mut options = CopyOptions::new();
    options.copy_inside = true; // Copies the contents of the folder rather than the folder itself

    // Remove the files in the list that are already in the destination folder if the complete flag is set
    if option == CopyPossiblesOptions::Complete {
        dir_content.files.retain(|item| {
            let destination_file =
                Path::new(destination_path).join(match Path::new(item).strip_prefix(source_path) {
                    Ok(rel_path) => rel_path,
                    Err(_) => {
                        eprintln!("Impossible to determine relative path for {:?}", item);
                        return false;
                    }
                });

            !destination_file.exists()
        });
    } else if option == CopyPossiblesOptions::Update {
        // Remove the files in the list that are already in the destination folder and are older than the source files
        dir_content.files.retain(|item| {
            let destination_file =
                Path::new(destination_path).join(match Path::new(item).strip_prefix(source_path) {
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

    // Create a progress bar for the copying of the files
    let pb_copy = m.add(ProgressBar::new(dir_content.files.len() as u64));
    pb_copy.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    // Wrap the progress bar to handle parallel iterations
    let pb_copy = Arc::new(pb_copy);

    // Start a thread to handle the progress bar
    let pb_copy_clone = Arc::clone(&pb_copy);
    let ticker = thread::spawn(move || {
        while !pb_copy_clone.is_finished() {
            pb_copy_clone.tick();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Parallel copying
    dir_content.files.par_iter().for_each(|item| {
        let pb_copy = Arc::clone(&pb_copy);

        let relative_path = match Path::new(item).strip_prefix(source_path) {
            Ok(rel_path) => rel_path,
            Err(_) => {
                eprintln!("Impossible to determine relative path for {:?}", item);
                return;
            }
        };

        let destination_file = Path::new(destination_path).join(relative_path);

        // Creates the necessary folders in the destination
        if let Some(parent) = destination_file.parent() {
            if let Err(_) = create_dir_all(parent) {
                eprintln!("Unable to create parent folders for {:?}", destination_file);
                return;
            }
        }

        // Make the copy
        if let Err(e) = copy(item, &destination_file) {
            eprintln!(
                "Error copying file {:?} to {:?}: {:?}",
                item, destination_file, e
            );
            return;
        }

        // Checks the hash of the copied file
        let source_hash = match calculate_hash(&Path::new(item)) {
            Ok(hash) => hash,
            Err(_) => {
                eprintln!("Error calculating source file hash for {:?}", item);
                return;
            }
        };

        let destination_hash = match calculate_hash(&destination_file) {
            Ok(hash) => hash,
            Err(_) => {
                eprintln!(
                    "Error calculating destination file hash for {:?}",
                    destination_file
                );
                return;
            }
        };

        if source_hash != destination_hash {
            eprintln!(
                "Hashes do not match for the files {:?} and {:?}",
                item, destination_file
            );
        }

        pb_copy.inc(1);
    });

    pb_copy.finish_with_message("Files copied successfully");

    // Wait for the ticker thread to finish
    ticker.join().unwrap();
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
    },
}

fn main() {
    let args = ArgsCli::parse();

    match args.command {
        Commands::Copy {
            source,
            destination,
            options:
                ArgsCopyPossiblesOptions {
                    replace,
                    complete,
                    update,
                },
        } => {
            let option = match (replace, complete, update) {
                (true, false, false) => CopyPossiblesOptions::Replace,
                (false, true, false) => CopyPossiblesOptions::Complete,
                (false, false, true) => CopyPossiblesOptions::Update,
                _ => CopyPossiblesOptions::None,
            };

            do_copy(&source, &destination, option);
        }
    }
}
