use crate::progress_bar_helper::ProgressBarHelper;

use super::{Node, Tree};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Result, Seek},
    path::{Path, PathBuf},
};

pub trait Verifyable {
    fn verify(&mut self, into: bool, files_to_verify: Vec<PathBuf>) -> Result<()>;
}

impl Verifyable for Tree {
    fn verify(&mut self, into: bool, files_to_verify: Vec<PathBuf>) -> Result<()> {
        self.src_root.lock()?;

        let result = self.src_root.verify(
            &self.src_root_path,
            &self.dest_root_path,
            into,
            files_to_verify,
        );

        self.src_root.unlock()?;

        result
    }
}

impl Node {
    fn verify(
        &self,
        source: &Path,
        destination: &Path,
        into: bool,
        files_to_verify: Vec<PathBuf>,
    ) -> Result<()> {
        let mut stack = if into {
            // Stack is initialized with the current node and the destination path
            vec![(self, source.to_path_buf(), destination.to_path_buf())]
        } else {
            // Stack is initialized with the children nodes and their destination path of the current node
            match self {
                Node::Folder(folder) => folder
                    .children
                    .par_iter()
                    .map(|child| (child, source.to_path_buf(), destination.to_path_buf()))
                    .collect(),
                _ => vec![],
            }
        };

        let mut files_stack = Vec::new();

        while let Some((node, src_path, dest_path)) = stack.pop() {
            let src_full_path = node.get_full_path(&src_path);
            let dest_full_path = node.get_full_path(&dest_path);

            match node {
                Node::File(file_node) => {
                    // Only verify the files that are in the list
                    if files_to_verify.contains(&dest_full_path) {
                        files_stack.push((
                            file_node,
                            src_full_path.clone(),
                            dest_full_path.clone(),
                        ));
                    }
                }
                Node::Folder(folder) => {
                    for child in &folder.children {
                        stack.push((child, src_full_path.clone(), dest_full_path.clone()));
                    }
                }
            }
        }

        let pb_verify = ProgressBarHelper::new(files_stack.len() as u64);

        pb_verify.set_message("Verifying files");

        files_stack
            .par_iter()
            .for_each(|(file_node, src_full_path, dest_full_path)| {
                if !dest_full_path.exists() {
                    // TODO: handle errors (info) here
                    println!("File not found: {:?}", dest_full_path);
                    return;
                }

                let mut src_file = &file_node.handle;
                // Reset the cursor to the beginning of the file
                match src_file.seek(std::io::SeekFrom::Start(0)) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Error seeking source file: {:?}", src_full_path);
                        return;
                    }
                };
                let mut src_hasher = Sha256::new();
                let mut src_buffer = Vec::new();
                match src_file.read_to_end(&mut src_buffer) {
                    Ok(_) => {}
                    Err(_) => {
                        // TODO: handle errors (info) here
                        println!("Error reading source file: {:?}", src_full_path);
                        return;
                    }
                };
                src_hasher.update(&src_buffer);
                let src_hash = src_hasher.finalize().to_vec();

                let mut dest_file = match File::open(&dest_full_path) {
                    Ok(file) => file,
                    Err(_) => {
                        // TODO: handle errors (info) here
                        println!("Error opening file: {:?}", dest_full_path);
                        return;
                    }
                };
                let mut dest_hasher = Sha256::new();
                let mut dest_buffer = Vec::new();
                match dest_file.read_to_end(&mut dest_buffer) {
                    Ok(_) => {}
                    Err(_) => {
                        // TODO: handle errors (info) here
                        println!("Error reading destination file: {:?}", dest_full_path);
                        return;
                    }
                };
                dest_hasher.update(&dest_buffer);
                let dest_hash = dest_hasher.finalize().to_vec();

                if src_hash != dest_hash {
                    // TODO: handle errors (info) here
                    println!("Hash mismatch: {:?} -> {:?}", src_full_path, dest_full_path);
                    return;
                } else {
                    pb_verify.inc(1);
                }
            });

        pb_verify.finish_with_message("Files verified");

        Ok(())
    }
}
