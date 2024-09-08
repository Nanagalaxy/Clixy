use super::{FileNode, Node, Tree};
use crate::commands::copy::CopyTypesOptions;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::OpenOptions,
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Add functionality to copy the content of a tree.
pub trait Copyable {
    fn copy(
        &mut self,
        into: bool,
        option: CopyTypesOptions,
        only_folders: bool,
    ) -> Result<Vec<PathBuf>>;
}

impl Copyable for Tree {
    /// Copy the content of the source path to the destination path.
    /// If `into` is `true`, the source path will be copied directly into the destination path.
    /// Returns a vector with the destination paths of the copied files.
    fn copy(
        &mut self,
        into: bool,
        option: CopyTypesOptions,
        only_folders: bool,
    ) -> Result<Vec<PathBuf>> {
        // Check if the destination path is empty if none option is set
        if option == CopyTypesOptions::None {
            let is_empty = !self.dest_root_path.exists()
                || (self.dest_root_path.is_dir()
                    && self.dest_root_path.read_dir()?.next().is_none());

            if !is_empty {
                eprintln!("Destination folder is not empty, please provide an empty folder or use an option");
                return Err(Error::new(
                    ErrorKind::AlreadyExists,
                    "Destination folder is not empty",
                ));
            }
        }

        self.src_root.lock()?;

        let result = Node::copy(
            &self.src_root,
            &self.dest_root_path,
            into,
            only_folders,
            option,
        );

        self.src_root.unlock()?;

        result
    }
}

// Implement all the copy logic for the nodes.
impl Node {
    /// Prepare the stack for the content of the tree.
    /// This will create the destination directory structure and add the file nodes to the stack.
    fn prepare_stack(&self, destination: &Path, into: bool) -> Result<Vec<(&FileNode, PathBuf)>> {
        // This stack will hold the nodes to be processed
        let mut stack = if into {
            // Stack is initialized with the current node and the destination path
            vec![(self, destination.to_path_buf())]
        } else {
            // Stack is initialized with the children nodes and their destination path of the current node
            match self {
                Node::Folder(folder) => folder
                    .children
                    .par_iter()
                    .map(|child| (child, destination.to_path_buf()))
                    .collect(),
                _ => vec![],
            }
        };

        // This stack will hold the file nodes and their destination path
        let mut files_stack = Vec::new();

        while let Some((node, dest_path)) = stack.pop() {
            let full_path = node.get_full_path(&dest_path);

            match node {
                Node::File(file_node) => {
                    files_stack.push((file_node, full_path));
                }
                Node::Folder(folder) => {
                    std::fs::create_dir_all(&full_path)?;

                    for child in &folder.children {
                        stack.push((child, full_path.clone()));
                    }
                }
            }
        }

        Ok(files_stack)
    }

    fn handle_copy_option(
        file_node: &FileNode,
        full_path: &Path,
        option: &CopyTypesOptions,
        open_options: &mut OpenOptions,
    ) -> Result<bool> {
        match option {
            CopyTypesOptions::None => {
                open_options.write(true).create_new(true);
                Ok(true)
            }
            CopyTypesOptions::Replace => {
                open_options.write(true).create(true).truncate(true);
                Ok(true)
            }
            CopyTypesOptions::Complete => {
                if full_path.exists() {
                    Ok(false)
                } else {
                    open_options.write(true).create_new(true);
                    Ok(true)
                }
            }
            CopyTypesOptions::Update => {
                if full_path.exists() {
                    let src_meta = file_node.handle.metadata()?;
                    let dest_meta = std::fs::metadata(full_path)?;

                    if src_meta.modified()? > dest_meta.modified()? {
                        open_options.write(true).create(true).truncate(true);
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    open_options.write(true).create_new(true);
                    Ok(true)
                }
            }
        }
    }

    fn copy(
        &self,
        destination: &Path,
        into: bool,
        only_folders: bool,
        option: CopyTypesOptions,
    ) -> Result<Vec<PathBuf>> {
        let files_stack = self.prepare_stack(destination, into)?;

        // Return early if we only want to copy folders and not files
        if only_folders {
            // XXX: files_stack are not used in this case, maybe we can avoid creating it in the first place
            return Ok(vec![]);
        }

        let copied_files: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));

        files_stack.par_iter().for_each(|(file_node, full_path)| {
            let mut open_options = OpenOptions::new();
            let do_copy = match Node::handle_copy_option(
                &file_node,
                &full_path,
                &option,
                &mut open_options,
            ) {
                Ok(do_copy) => do_copy,
                Err(_) => {
                    // TODO: handle errors (info) here
                    return;
                }
            };

            if do_copy {
                let mut dest_file = match open_options.open(full_path) {
                    Ok(file) => file,
                    Err(_) => {
                        // TODO: handle errors (info) here
                        return;
                    }
                };

                match std::io::copy(&mut &file_node.handle, &mut dest_file) {
                    Ok(_) => {
                        // TODO: update progress bar here
                        match copied_files.lock() {
                            Ok(mut copied_files) => copied_files.push(full_path.clone()),
                            Err(_) => {
                                // TODO: handle errors (info) here
                                return;
                            }
                        }
                    }
                    Err(_) => {
                        // TODO: handle errors (info) here
                        return;
                    }
                };
            } else {
                // TODO: update progress bar here
            }
        });

        let copied_files = match Arc::into_inner(copied_files) {
            Some(copied_files) => copied_files.into_inner().unwrap_or(Vec::new()),
            None => {
                eprintln!("Error getting copied files");
                return Err(Error::new(ErrorKind::Other, "Error getting copied files"));
            }
        };

        Ok(copied_files)
    }
}
