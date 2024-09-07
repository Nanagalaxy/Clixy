use fs4::fs_std::FileExt;
use std::{
    fs::File,
    io::Result,
    path::{Path, PathBuf},
};

/// A file node in the tree.
pub struct FileNode {
    /// The file name without the extension
    name: String,

    /// The file handle
    pub handle: File,

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
pub struct FolderNode {
    /// The folder name
    name: String,

    /// The children nodes (files or folders)
    pub children: Vec<Node>,
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
pub enum Node {
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
    pub fn lock(&mut self) -> Result<()> {
        match self {
            Node::File(file) => file.lock(),
            Node::Folder(folder) => folder.lock(),
        }
    }

    /// Unlock a file node or all children nodes of a folder node.
    pub fn unlock(&mut self) -> Result<()> {
        match self {
            Node::File(file) => file.unlock(),
            Node::Folder(folder) => folder.unlock(),
        }
    }

    pub fn get_full_path(&self, parent_path: &Path) -> PathBuf {
        match self {
            Node::File(file) => parent_path.join(format!("{}.{}", file.name, file.extension)),
            Node::Folder(folder) => parent_path.join(&folder.name),
        }
    }
}

/// Struct to hold a tree of files and folders
pub struct Tree {
    pub src_root: Node,

    /// The path of the destination root node
    pub dest_root_path: PathBuf,
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
}
