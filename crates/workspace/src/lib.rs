//! Workspace management for the AI code editor.
//!
//! Provides file tree building, file operations, file watching,
//! and workspace settings persistence.

pub mod ops;
pub mod settings;
pub mod tree;
pub mod watcher;

pub use ops::{FileMetadata, FileOpError, FileOpResult, FileOps};
pub use settings::{GlobalSettings, WorkspaceSettings};
pub use tree::{FlatTreeItem, NodeKind, TreeNode};
pub use watcher::{FileWatcher, WatchEvent};

use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Main workspace service that coordinates file tree, operations, and watching.
#[derive(Debug)]
pub struct WorkspaceService {
    /// Root path of the workspace
    root: PathBuf,
    /// File tree cache
    tree: Option<TreeNode>,
    /// File watcher
    watcher: Option<FileWatcher>,
    /// Workspace settings
    settings: WorkspaceSettings,
}

impl WorkspaceService {
    /// Open a folder as a workspace.
    pub fn open(root: PathBuf) -> Result<Self, String> {
        if !root.exists() {
            return Err(format!("path does not exist: {}", root.display()));
        }
        if !root.is_dir() {
            return Err(format!("path is not a directory: {}", root.display()));
        }

        let settings = WorkspaceSettings::load(&root)
            .unwrap_or_else(|| WorkspaceSettings::new(root.clone()));

        // Update global recent workspaces
        let mut global = GlobalSettings::load();
        global.add_recent_workspace(root.clone());
        let _ = global.save();

        Ok(Self {
            root,
            tree: None,
            watcher: None,
            settings,
        })
    }

    /// Get the workspace root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the workspace name (directory name).
    pub fn name(&self) -> &str {
        self.root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("workspace")
    }

    /// Build or refresh the file tree.
    pub fn build_tree(&mut self) -> &TreeNode {
        let mut root_node = TreeNode::directory(self.root.clone());
        root_node.expanded = true;

        // Use ignore crate to respect .gitignore
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        let mut paths: Vec<PathBuf> = walker
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .filter(|p| p != &self.root)
            .collect();

        paths.sort();

        for path in paths {
            self.insert_path(&mut root_node, &path);
        }

        root_node.sort_children();

        // Restore expanded state from settings
        for expanded_path in &self.settings.expanded_dirs {
            if let Some(node) = root_node.find_by_path_mut(expanded_path) {
                node.expanded = true;
            }
        }

        self.tree = Some(root_node);
        self.tree.as_ref().unwrap()
    }

    /// Insert a path into the tree.
    fn insert_path(&self, root: &mut TreeNode, path: &Path) {
        let relative = match path.strip_prefix(&self.root) {
            Ok(r) => r,
            Err(_) => return,
        };

        let components: Vec<_> = relative.components().collect();
        let mut current = root;

        for (i, component) in components.iter().enumerate() {
            let name = component.as_os_str().to_string_lossy().to_string();
            let full_path = self.root.join(
                components[..=i]
                    .iter()
                    .map(|c| c.as_os_str())
                    .collect::<PathBuf>(),
            );

            let is_last = i == components.len() - 1;
            let is_dir = if is_last {
                path.is_dir()
            } else {
                true
            };

            let existing_idx = current.children.iter().position(|c| c.name == name);

            if let Some(idx) = existing_idx {
                current = &mut current.children[idx];
            } else {
                let node = if is_dir {
                    TreeNode::directory(full_path)
                } else {
                    TreeNode::file(full_path)
                };
                current.children.push(node);
                let idx = current.children.len() - 1;
                current = &mut current.children[idx];
            }
        }
    }

    /// Get the cached file tree.
    pub fn tree(&self) -> Option<&TreeNode> {
        self.tree.as_ref()
    }

    /// Get a flattened view of the tree for UI rendering.
    pub fn flat_tree(&self) -> Vec<FlatTreeItem> {
        self.tree
            .as_ref()
            .map(|t| FlatTreeItem::flatten_tree(t, false))
            .unwrap_or_default()
    }

    /// Toggle directory expansion.
    pub fn toggle_expand(&mut self, path: &Path) {
        if let Some(tree) = &mut self.tree {
            if let Some(node) = tree.find_by_path_mut(path) {
                if node.is_directory() {
                    node.expanded = !node.expanded;
                    self.update_expanded_dirs();
                }
            }
        }
    }

    /// Update expanded dirs in settings.
    fn update_expanded_dirs(&mut self) {
        if let Some(tree) = &self.tree {
            let expanded: Vec<PathBuf> = self.collect_expanded(tree);
            self.settings.set_expanded_dirs(expanded);
        }
    }

    /// Collect all expanded directory paths.
    fn collect_expanded(&self, node: &TreeNode) -> Vec<PathBuf> {
        let mut result = Vec::new();
        if node.is_directory() && node.expanded {
            result.push(node.path.clone());
            for child in &node.children {
                result.extend(self.collect_expanded(child));
            }
        }
        result
    }

    /// Start file watching.
    pub fn start_watching(&mut self) -> Result<(), String> {
        let watcher = FileWatcher::new(&self.root)?;
        self.watcher = Some(watcher);
        Ok(())
    }

    /// Get watch event receiver.
    pub fn watch_events(&self) -> Option<tokio::sync::broadcast::Receiver<WatchEvent>> {
        self.watcher.as_ref().map(|w| w.subscribe())
    }

    /// Get workspace settings.
    pub fn settings(&self) -> &WorkspaceSettings {
        &self.settings
    }

    /// Get mutable workspace settings.
    pub fn settings_mut(&mut self) -> &mut WorkspaceSettings {
        &mut self.settings
    }

    /// Save workspace settings.
    pub fn save_settings(&self) -> Result<(), String> {
        self.settings.save()
    }

    /// Create a new file in the workspace.
    pub fn create_file(&mut self, path: &Path, content: Option<&str>) -> FileOpResult<()> {
        FileOps::create_file(path, content)?;
        self.build_tree();
        Ok(())
    }

    /// Create a new directory in the workspace.
    pub fn create_directory(&mut self, path: &Path) -> FileOpResult<()> {
        FileOps::create_directory(path)?;
        self.build_tree();
        Ok(())
    }

    /// Rename a file or directory.
    pub fn rename(&mut self, from: &Path, to: &Path) -> FileOpResult<()> {
        FileOps::rename(from, to)?;
        self.build_tree();
        Ok(())
    }

    /// Delete a file.
    pub fn delete_file(&mut self, path: &Path) -> FileOpResult<()> {
        FileOps::delete_file(path)?;
        self.build_tree();
        Ok(())
    }

    /// Delete a directory.
    pub fn delete_directory(&mut self, path: &Path) -> FileOpResult<()> {
        FileOps::delete_directory(path)?;
        self.build_tree();
        Ok(())
    }
}
