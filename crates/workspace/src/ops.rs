//! File operations for workspace management.

use std::fs;
use std::path::{Path, PathBuf};

/// Result type for file operations.
pub type FileOpResult<T> = Result<T, FileOpError>;

/// Errors that can occur during file operations.
#[derive(Debug, Clone)]
pub enum FileOpError {
    /// File or directory not found
    NotFound(PathBuf),
    /// File or directory already exists
    AlreadyExists(PathBuf),
    /// Permission denied
    PermissionDenied(PathBuf),
    /// Invalid path or name
    InvalidPath(String),
    /// IO error
    IoError(String),
}

impl std::fmt::Display for FileOpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOpError::NotFound(p) => write!(f, "not found: {}", p.display()),
            FileOpError::AlreadyExists(p) => write!(f, "already exists: {}", p.display()),
            FileOpError::PermissionDenied(p) => write!(f, "permission denied: {}", p.display()),
            FileOpError::InvalidPath(s) => write!(f, "invalid path: {s}"),
            FileOpError::IoError(s) => write!(f, "IO error: {s}"),
        }
    }
}

impl std::error::Error for FileOpError {}

impl From<std::io::Error> for FileOpError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => FileOpError::NotFound(PathBuf::new()),
            std::io::ErrorKind::AlreadyExists => FileOpError::AlreadyExists(PathBuf::new()),
            std::io::ErrorKind::PermissionDenied => FileOpError::PermissionDenied(PathBuf::new()),
            _ => FileOpError::IoError(e.to_string()),
        }
    }
}

/// File operations handler.
#[derive(Debug, Clone)]
pub struct FileOps;

impl FileOps {
    /// Create a new file with optional initial content.
    pub fn create_file(path: &Path, content: Option<&str>) -> FileOpResult<()> {
        if path.exists() {
            return Err(FileOpError::AlreadyExists(path.to_path_buf()));
        }
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| FileOpError::IoError(e.to_string()))?;
            }
        }
        let content = content.unwrap_or("");
        fs::write(path, content).map_err(|e| FileOpError::IoError(e.to_string()))
    }

    /// Create a new directory.
    pub fn create_directory(path: &Path) -> FileOpResult<()> {
        if path.exists() {
            return Err(FileOpError::AlreadyExists(path.to_path_buf()));
        }
        fs::create_dir_all(path).map_err(|e| FileOpError::IoError(e.to_string()))
    }

    /// Rename a file or directory.
    pub fn rename(from: &Path, to: &Path) -> FileOpResult<()> {
        if !from.exists() {
            return Err(FileOpError::NotFound(from.to_path_buf()));
        }
        if to.exists() {
            return Err(FileOpError::AlreadyExists(to.to_path_buf()));
        }
        fs::rename(from, to).map_err(|e| FileOpError::IoError(e.to_string()))
    }

    /// Delete a file.
    pub fn delete_file(path: &Path) -> FileOpResult<()> {
        if !path.exists() {
            return Err(FileOpError::NotFound(path.to_path_buf()));
        }
        if !path.is_file() {
            return Err(FileOpError::InvalidPath("not a file".to_string()));
        }
        fs::remove_file(path).map_err(|e| FileOpError::IoError(e.to_string()))
    }

    /// Delete a directory and all its contents.
    pub fn delete_directory(path: &Path) -> FileOpResult<()> {
        if !path.exists() {
            return Err(FileOpError::NotFound(path.to_path_buf()));
        }
        if !path.is_dir() {
            return Err(FileOpError::InvalidPath("not a directory".to_string()));
        }
        fs::remove_dir_all(path).map_err(|e| FileOpError::IoError(e.to_string()))
    }

    /// Copy a file.
    pub fn copy_file(from: &Path, to: &Path) -> FileOpResult<()> {
        if !from.exists() {
            return Err(FileOpError::NotFound(from.to_path_buf()));
        }
        if to.exists() {
            return Err(FileOpError::AlreadyExists(to.to_path_buf()));
        }
        if let Some(parent) = to.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| FileOpError::IoError(e.to_string()))?;
            }
        }
        fs::copy(from, to).map_err(|e| FileOpError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Read file contents as string.
    pub fn read_file(path: &Path) -> FileOpResult<String> {
        if !path.exists() {
            return Err(FileOpError::NotFound(path.to_path_buf()));
        }
        fs::read_to_string(path).map_err(|e| FileOpError::IoError(e.to_string()))
    }

    /// Write content to file.
    pub fn write_file(path: &Path, content: &str) -> FileOpResult<()> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| FileOpError::IoError(e.to_string()))?;
            }
        }
        fs::write(path, content).map_err(|e| FileOpError::IoError(e.to_string()))
    }

    /// Check if path exists.
    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    /// Check if path is a file.
    pub fn is_file(path: &Path) -> bool {
        path.is_file()
    }

    /// Check if path is a directory.
    pub fn is_directory(path: &Path) -> bool {
        path.is_dir()
    }

    /// Get file metadata.
    pub fn metadata(path: &Path) -> FileOpResult<FileMetadata> {
        let meta = fs::metadata(path).map_err(|e| FileOpError::IoError(e.to_string()))?;
        Ok(FileMetadata {
            size: meta.len(),
            is_readonly: meta.permissions().readonly(),
            modified: meta.modified().ok(),
        })
    }
}

/// File metadata.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: u64,
    /// Whether file is read-only
    pub is_readonly: bool,
    /// Last modified time
    pub modified: Option<std::time::SystemTime>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_and_delete_file() {
        let temp_dir = std::env::temp_dir().join("workspace_test_file");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.txt");
        FileOps::create_file(&file_path, Some("hello")).unwrap();
        assert!(file_path.exists());

        let content = FileOps::read_file(&file_path).unwrap();
        assert_eq!(content, "hello");

        FileOps::delete_file(&file_path).unwrap();
        assert!(!file_path.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
