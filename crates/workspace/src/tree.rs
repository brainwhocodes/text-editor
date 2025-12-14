//! File tree data structures for workspace exploration.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Represents a node in the file tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    /// File or directory name
    pub name: String,
    /// Full path to the file or directory
    pub path: PathBuf,
    /// Type of the node
    pub kind: NodeKind,
    /// Children nodes (only for directories)
    pub children: Vec<TreeNode>,
    /// Whether this directory is expanded in the UI
    pub expanded: bool,
}

/// Type of tree node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    File,
    Directory,
}

impl TreeNode {
    /// Create a new file node.
    pub fn file(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        Self {
            name,
            path,
            kind: NodeKind::File,
            children: Vec::new(),
            expanded: false,
        }
    }

    /// Create a new directory node.
    pub fn directory(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        Self {
            name,
            path,
            kind: NodeKind::Directory,
            children: Vec::new(),
            expanded: false,
        }
    }

    /// Check if this is a file.
    pub fn is_file(&self) -> bool {
        self.kind == NodeKind::File
    }

    /// Check if this is a directory.
    pub fn is_directory(&self) -> bool {
        self.kind == NodeKind::Directory
    }

    /// Get the file extension if this is a file.
    pub fn extension(&self) -> Option<&str> {
        if self.is_file() {
            self.path.extension().and_then(|s| s.to_str())
        } else {
            None
        }
    }

    /// Sort children: directories first, then files, alphabetically.
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| {
            match (&a.kind, &b.kind) {
                (NodeKind::Directory, NodeKind::File) => std::cmp::Ordering::Less,
                (NodeKind::File, NodeKind::Directory) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        for child in &mut self.children {
            child.sort_children();
        }
    }

    /// Find a node by path.
    pub fn find_by_path(&self, target: &Path) -> Option<&TreeNode> {
        if self.path == target {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_path(target) {
                return Some(found);
            }
        }
        None
    }

    /// Find a mutable node by path.
    pub fn find_by_path_mut(&mut self, target: &Path) -> Option<&mut TreeNode> {
        if self.path == target {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_by_path_mut(target) {
                return Some(found);
            }
        }
        None
    }

    /// Count total nodes in tree.
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }

    /// Flatten the tree to a list of paths.
    pub fn flatten(&self) -> Vec<&PathBuf> {
        let mut result = vec![&self.path];
        for child in &self.children {
            result.extend(child.flatten());
        }
        result
    }
}

/// A flattened view of the tree for UI rendering.
#[derive(Debug, Clone)]
pub struct FlatTreeItem {
    /// The tree node
    pub node: TreeNode,
    /// Depth level (0 = root)
    pub depth: usize,
    /// Whether this item is visible (parent expanded)
    pub visible: bool,
}

impl FlatTreeItem {
    /// Flatten a tree into a list suitable for UI rendering.
    pub fn flatten_tree(root: &TreeNode, include_root: bool) -> Vec<FlatTreeItem> {
        let mut result = Vec::new();
        if include_root {
            Self::flatten_recursive(root, 0, true, &mut result);
        } else {
            for child in &root.children {
                Self::flatten_recursive(child, 0, true, &mut result);
            }
        }
        result
    }

    fn flatten_recursive(
        node: &TreeNode,
        depth: usize,
        visible: bool,
        result: &mut Vec<FlatTreeItem>,
    ) {
        result.push(FlatTreeItem {
            node: node.clone(),
            depth,
            visible,
        });
        if node.is_directory() {
            let child_visible = visible && node.expanded;
            for child in &node.children {
                Self::flatten_recursive(child, depth + 1, child_visible, result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_creation() {
        let file = TreeNode::file(PathBuf::from("/test/file.rs"));
        assert!(file.is_file());
        assert_eq!(file.name, "file.rs");
        assert_eq!(file.extension(), Some("rs"));

        let dir = TreeNode::directory(PathBuf::from("/test/src"));
        assert!(dir.is_directory());
        assert_eq!(dir.name, "src");
    }

    #[test]
    fn test_sort_children() {
        let mut root = TreeNode::directory(PathBuf::from("/test"));
        root.children = vec![
            TreeNode::file(PathBuf::from("/test/z.rs")),
            TreeNode::directory(PathBuf::from("/test/src")),
            TreeNode::file(PathBuf::from("/test/a.rs")),
        ];
        root.sort_children();
        assert_eq!(root.children[0].name, "src");
        assert_eq!(root.children[1].name, "a.rs");
        assert_eq!(root.children[2].name, "z.rs");
    }
}
