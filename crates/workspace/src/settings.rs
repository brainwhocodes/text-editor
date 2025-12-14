//! Workspace settings and persistence.

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Workspace-level settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    /// Root path of the workspace
    pub root: PathBuf,
    /// Recently opened files (most recent first)
    pub recent_files: Vec<PathBuf>,
    /// Last open tabs when workspace was closed
    pub last_open_tabs: Vec<PathBuf>,
    /// Active tab index
    pub active_tab_index: Option<usize>,
    /// Expanded directories in explorer
    pub expanded_dirs: Vec<PathBuf>,
}

impl WorkspaceSettings {
    /// Create new settings for a workspace root.
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            recent_files: Vec::new(),
            last_open_tabs: Vec::new(),
            active_tab_index: None,
            expanded_dirs: Vec::new(),
        }
    }

    /// Add a file to recent files list.
    pub fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 20 {
            self.recent_files.truncate(20);
        }
    }

    /// Update open tabs.
    pub fn set_open_tabs(&mut self, tabs: Vec<PathBuf>, active: Option<usize>) {
        self.last_open_tabs = tabs;
        self.active_tab_index = active;
    }

    /// Update expanded directories.
    pub fn set_expanded_dirs(&mut self, dirs: Vec<PathBuf>) {
        self.expanded_dirs = dirs;
    }

    /// Get settings file path for a workspace.
    fn settings_path(root: &Path) -> Option<PathBuf> {
        let dirs = ProjectDirs::from("dev", "text_editor", "ai_code_editor")?;
        let hash = Self::hash_path(root);
        Some(dirs.data_dir().join("workspaces").join(format!("{hash}.json")))
    }

    /// Hash a path to a filename-safe string.
    fn hash_path(path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Load settings from disk.
    pub fn load(root: &Path) -> Option<Self> {
        let path = Self::settings_path(root)?;
        let data = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Save settings to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::settings_path(&self.root).ok_or("no settings path")?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}

/// Global application settings (across all workspaces).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalSettings {
    /// Recently opened workspaces (most recent first)
    pub recent_workspaces: Vec<PathBuf>,
}

impl GlobalSettings {
    /// Add a workspace to recent list.
    pub fn add_recent_workspace(&mut self, root: PathBuf) {
        self.recent_workspaces.retain(|p| p != &root);
        self.recent_workspaces.insert(0, root);
        if self.recent_workspaces.len() > 10 {
            self.recent_workspaces.truncate(10);
        }
    }

    /// Get global settings file path.
    fn settings_path() -> Option<PathBuf> {
        let dirs = ProjectDirs::from("dev", "text_editor", "ai_code_editor")?;
        Some(dirs.data_dir().join("global_settings.json"))
    }

    /// Load global settings.
    pub fn load() -> Self {
        Self::settings_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save global settings.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::settings_path().ok_or("no settings path")?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_files() {
        let mut settings = WorkspaceSettings::new(PathBuf::from("/test"));
        settings.add_recent_file(PathBuf::from("/test/a.rs"));
        settings.add_recent_file(PathBuf::from("/test/b.rs"));
        settings.add_recent_file(PathBuf::from("/test/a.rs")); // duplicate

        assert_eq!(settings.recent_files.len(), 2);
        assert_eq!(settings.recent_files[0], PathBuf::from("/test/a.rs"));
    }
}
