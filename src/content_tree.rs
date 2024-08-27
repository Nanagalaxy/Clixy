use fs4::fs_std::FileExt;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::File,
    io::Result,
    path::{Path, PathBuf},
};

/// A file node in the tree.
struct FileNode {
    /// The file name without the extension
    name: String,

    /// The file handle
    handle: File,

    /// The file extension
    extension: String,

    /// If the file is locked
    is_locked: bool,
}

impl FileNode {
    fn new(name: String, handle: File, extension: String) -> Self {
        Self {
            name,
            handle,
            extension,
            is_locked: false,
        }
    }

    /// Lock the file
    fn lock(&mut self) -> Result<()> {
        if !self.is_locked {
            self.handle.lock_exclusive()?;
            self.is_locked = true;
        }

        Ok(())
    }

    /// Unlock the file
    fn unlock(&mut self) -> Result<()> {
        if self.is_locked {
            self.handle.unlock()?;
            self.is_locked = false;
        }

        Ok(())
    }
}

/// A folder node in the tree.
struct FolderNode {
    /// The folder name
    name: String,

    /// The children nodes (files or folders)
    children: Vec<Node>,
}

impl FolderNode {
    fn new(name: String, children: Vec<Node>) -> Self {
        Self { name, children }
    }

    /// Lock all children nodes.
    fn lock(&mut self) -> Result<()> {
        let mut stack = vec![self];

        while let Some(folder) = stack.pop() {
            for child in &mut folder.children {
                match child {
                    Node::File(file_node) => file_node.lock()?,
                    Node::Folder(folder_node) => stack.push(folder_node),
                }
            }
        }

        Ok(())
    }

    /// Unlock all children nodes.
    fn unlock(&mut self) -> Result<()> {
        let mut stack = vec![self];

        while let Some(folder) = stack.pop() {
            for child in &mut folder.children {
                match child {
                    Node::File(file_node) => file_node.unlock()?,
                    Node::Folder(folder_node) => stack.push(folder_node),
                }
            }
        }

        Ok(())
    }
}

/// A node in the tree. This can be a file or a folder.
enum Node {
    File(FileNode),
    Folder(FolderNode),
}

impl Node {
    /// Create a new node from a path.
    /// Note: This will return None if an error occurs. For example, if the path does not exist
    /// or if the path is not a file or folder (terminates with `..` for example).
    /// If a folder is provided, the children nodes will be created recursively.
    fn new(path: &Path) -> Option<Self> {
        if path.is_file() {
            let name = path.file_stem()?.to_string_lossy().to_string();
            let handle = File::open(path).ok()?;
            let extension = path.extension()?.to_string_lossy().to_string();

            Some(Node::File(FileNode::new(name, handle, extension)))
        } else if path.is_dir() {
            let name = path.file_name()?.to_string_lossy().to_string();

            // Construct children nodes
            // If an error occurs in any of the children, return None
            let children = path
                .read_dir()
                .ok()?
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    Node::new(&path)
                })
                .collect();

            Some(Node::Folder(FolderNode::new(name, children)))
        } else {
            None
        }
    }

    /// Lock a file node or all children nodes of a folder node.
    fn lock(&mut self) -> Result<()> {
        match self {
            Node::File(file) => file.lock(),
            Node::Folder(folder) => folder.lock(),
        }
    }

    /// Unlock a file node or all children nodes of a folder node.
    fn unlock(&mut self) -> Result<()> {
        match self {
            Node::File(file) => file.unlock(),
            Node::Folder(folder) => folder.unlock(),
        }
    }

    fn get_full_path(&self, parent_path: &Path) -> PathBuf {
        match self {
            Node::File(file) => parent_path.join(format!("{}.{}", file.name, file.extension)),
            Node::Folder(folder) => parent_path.join(&folder.name),
        }
    }

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

    fn copy_mono(&self, destination: &Path, into: bool) -> Result<()> {
        let mut stack = if into {
            // Stack is initialized with the current node and the destination path
            vec![(self, destination.to_path_buf())]
        } else {
            // Stack is initialized with the children nodes and their destination path of the current node
            match self {
                Node::Folder(folder) => folder
                    .children
                    .iter()
                    .map(|child| (child, destination.to_path_buf()))
                    .collect(),
                _ => vec![],
            }
        };

        while let Some((node, dest_path)) = stack.pop() {
            let full_path = node.get_full_path(&dest_path);

            match node {
                Node::File(file) => {
                    let mut dest_file = File::create(&full_path)?;
                    std::io::copy(&mut &file.handle, &mut dest_file)?;
                }
                Node::Folder(folder) => {
                    std::fs::create_dir_all(&full_path)?;

                    for child in &folder.children {
                        stack.push((child, full_path.clone()));
                    }
                }
            }
        }

        Ok(())
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

/// Struct to hold a tree of files and folders
pub struct Tree {
    src_root: Node,

    /// The path of the destination root node
    dest_root_path: PathBuf,
}

impl Tree {
    /// Create a new tree from a source path and a destination path.
    pub fn new(source_root_path: &Path, destination_root_path: &Path) -> Option<Self> {
        let source_root = Node::new(source_root_path)?;

        Some(Self {
            src_root: source_root,
            dest_root_path: destination_root_path.to_path_buf(),
        })
    }

    /// Copy the content of the source path to the destination path.
    /// If `into` is `true`, the source path will be copied directly into the destination path.
    pub fn copy(&mut self, into: bool) -> Result<()> {
        self.src_root.lock()?;

        let result = Node::copy(&self.src_root, &self.dest_root_path, into);

        self.src_root.unlock()?;

        result
    }
}
