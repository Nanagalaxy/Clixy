use super::{FileNode, Node, Tree};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::File,
    io::Result,
    path::{Path, PathBuf},
};

/// Add functionality to copy the content of a tree.
pub trait Copyable {
    fn copy(&mut self, into: bool) -> Result<()>;
}

impl Copyable for Tree {
    /// Copy the content of the source path to the destination path.
    /// If `into` is `true`, the source path will be copied directly into the destination path.
    fn copy(&mut self, into: bool) -> Result<()> {
        self.src_root.lock()?;

        let result = Node::copy(&self.src_root, &self.dest_root_path, into);

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

    fn copy(&self, destination: &Path, into: bool) -> Result<()> {
        let files_stack = self.prepare_stack(destination, into)?;

        files_stack.par_iter().for_each(|(file_node, full_path)| {
            let mut dest_file = match File::create(full_path) {
                Ok(file) => file,
                Err(_) => {
                    // TODO: handle errors (info) here
                    return;
                }
            };

            match std::io::copy(&mut &file_node.handle, &mut dest_file) {
                Ok(_) => {
                    // TODO: update progress bar here
                }
                Err(_) => {
                    // TODO: handle errors (info) here
                    return;
                }
            };
        });

        Ok(())
    }
}
