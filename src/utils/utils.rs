use rand::distr::{Alphanumeric, SampleString};
use rand::rng;
use std::fs::{read_dir, File};
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Function to add an error to a list of errors
pub fn add_error(list_of_errors: &Arc<Mutex<Vec<String>>>, error: String) {
    if let Ok(mut errors) = list_of_errors.lock() {
        errors.push(error);
    } else {
        // TODO: What to do here?
    }
}

#[allow(dead_code)]
pub struct AllowedPermissions {
    /// Whether the file or folder is readable
    pub read: bool,

    /// Whether the file or folder is writable (and deletable)
    pub write: bool,
}

/// Function to check the permissions of a file or folder
#[allow(dead_code)]
pub fn check_permissions(path: &Path, test_write: bool) -> Result<AllowedPermissions> {
    let read_permission: bool;
    let write_permission: bool;

    // Create a random string to test write permissions
    let random_string = Alphanumeric.sample_string(&mut rng(), 20);

    // Check if path is a file or folder
    if path.is_dir() {
        // Try to read the folder to check if it's readable
        match read_dir(path) {
            Ok(_) => {
                read_permission = true;
            }
            Err(_) => {
                read_permission = false;
            }
        }

        if test_write && read_permission {
            // Try to create a file in the folder to check if it's writable
            let test_file = path.join(random_string);

            match File::create(&test_file) {
                Ok(_) => {
                    std::fs::remove_file(&test_file)?;
                    write_permission = true;
                }
                Err(_) => {
                    write_permission = false;
                }
            }
        } else {
            write_permission = false;
        }
    } else if path.is_file() {
        // Try to read the file to check if it's readable
        match File::open(path) {
            Ok(_) => {
                read_permission = true;
            }
            Err(_) => {
                read_permission = false;
            }
        }

        if test_write && read_permission {
            // Try to write to the parent folder to check if it's writable
            let Some(parent_folder) = path.parent() else {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    "Error getting parent folder",
                ));
            };

            let test_file = parent_folder.join(random_string);

            match File::create(&test_file) {
                Ok(_) => {
                    std::fs::remove_file(&test_file)?;
                    write_permission = true;
                }
                Err(_) => {
                    write_permission = false;
                }
            }
        } else {
            write_permission = false;
        }
    } else {
        return Err(Error::new(ErrorKind::NotFound, "The path doesn't exist"));
    }

    Ok(AllowedPermissions {
        read: read_permission,
        write: write_permission,
    })
}

/// Function to confirm if the user wants to continue with the operation.
/// Returns true if the user confirms, false otherwise
/// Defaults to false if the user doesn't input anything
pub fn confirm_continue() -> bool {
    println!("Do you want to continue? (y/N)");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap_or_default();

    input.trim().to_lowercase() == "y"
}

/// Function to round the size of a file or folder to a human-readable format
#[allow(clippy::cast_precision_loss)]
pub fn round_bytes_size(size: u64) -> String {
    let kb = 1024;
    let mb = kb * 1024;
    let gb = mb * 1024;
    let tb = gb * 1024;

    if size < kb {
        format!("{size} B")
    } else if size < mb {
        format!("{:.2} KB", size as f64 / kb as f64)
    } else if size < gb {
        format!("{:.2} MB", size as f64 / mb as f64)
    } else if size < tb {
        format!("{:.2} GB", size as f64 / gb as f64)
    } else {
        format!("{:.2} TB", size as f64 / tb as f64)
    }
}

#[test]
fn test_round_bytes_size() {
    assert_eq!(round_bytes_size(0), "0 B");
    assert_eq!(round_bytes_size(1023), "1023 B");
    assert_eq!(round_bytes_size(1024), "1.00 KB");
    assert_eq!(round_bytes_size(1024 * 1024), "1.00 MB");
    assert_eq!(round_bytes_size(1024 * 1024 * 1024), "1.00 GB");
    assert_eq!(round_bytes_size(1024 * 1024 * 1024 * 1024), "1.00 TB");
}
