use sha2::{Digest, Sha256};
use std::fs::{read_dir, File};
use std::io::{Error, ErrorKind, Read, Result};
use std::path::Path;

/// Function for calculating the SHA-256 hash of a file
pub fn calculate_hash(file_path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);

    Ok(hasher.finalize().to_vec())
}

/// Function to check the accessibility of a file or folder
pub fn check_accessibility(path: &Path) -> Result<()> {
    if path.is_file() {
        File::open(path).map(|_| ())
    } else if path.is_dir() {
        read_dir(path).map(|_| ())
    } else {
        Err(Error::new(ErrorKind::NotFound, "The path doesn't exist"))
    }
}
