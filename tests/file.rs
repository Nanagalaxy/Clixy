use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    process::Command,
};
use tempfile::tempdir;

#[test]
fn copy() {
    let src_dir = tempdir().unwrap();
    let src_path = src_dir.path();

    let dest_dir = tempdir().unwrap();
    let dest_path = dest_dir.path();

    let src_file_path = src_path.join("file.txt");
    let mut src_file = File::create(src_file_path).unwrap();
    src_file.write_all(b"Hello, world!").unwrap();

    let bin_path = Path::new(env!("CARGO_BIN_EXE_clixy"));

    // clixy file copy -s src_path -d dest_path
    Command::new(bin_path)
        .arg("file")
        .arg("copy")
        .arg("-s")
        .arg(src_path)
        .arg("-d")
        .arg(dest_path)
        .output()
        .expect("Failed to execute command");

    let dest_file_path = dest_path.join("file.txt");
    let mut dest_file = File::open(&dest_file_path).unwrap();
    let mut dest_content = String::new();
    dest_file.read_to_string(&mut dest_content).unwrap();

    assert_eq!(dest_content, "Hello, world!");

    drop(src_file);
    drop(dest_file);
    src_dir.close().unwrap();
    dest_dir.close().unwrap();
}

#[test]
fn remove() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    let file_path = path.join("file.txt");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"Hello, world!").unwrap();
    // Close the file to release the lock
    drop(file);

    println!("path: {path:?}");
    println!("file_path: {file_path:?}");

    let bin_path = Path::new(env!("CARGO_BIN_EXE_clixy"));

    // clixy file remove -s path -y --only-files
    Command::new(bin_path)
        .arg("file")
        .arg("remove")
        .arg("-s")
        .arg(path)
        .arg("-y")
        .arg("--only-files")
        .output()
        .expect("Failed to execute command");

    assert!(!file_path.exists());

    dir.close().unwrap();
}

#[test]
fn move_file() {
    let src_dir = tempdir().unwrap();
    let src_path = src_dir.path();

    let dest_dir = tempdir().unwrap();
    let dest_path = dest_dir.path();

    let src_file_path = src_path.join("file.txt");
    let mut src_file = File::create(&src_file_path).unwrap();
    src_file.write_all(b"Hello, world!").unwrap();
    drop(src_file);

    let dest_file_path = dest_path.join("file.txt");

    let bin_path = Path::new(env!("CARGO_BIN_EXE_clixy"));

    // clixy file move -s src_file_path -d dest_file_path
    Command::new(bin_path)
        .arg("file")
        .arg("move")
        .arg("-s")
        .arg(&src_file_path)
        .arg("-d")
        .arg(&dest_file_path)
        .output()
        .expect("Failed to execute command");

    assert!(!src_file_path.exists());

    assert!(dest_file_path.exists());

    let mut dest_file = File::open(dest_file_path).unwrap();
    let mut dest_content = String::new();
    dest_file.read_to_string(&mut dest_content).unwrap();

    assert_eq!(dest_content, "Hello, world!");

    drop(dest_file);
    src_dir.close().unwrap();
    dest_dir.close().unwrap();
}
