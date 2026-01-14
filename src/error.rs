//! Error types for mvln operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during mvln operations.
#[derive(Error, Debug)]
pub enum MvlnError {
    /// Source file or directory not found.
    #[error("source not found: {path}")]
    SourceNotFound { path: PathBuf },

    /// Destination already exists and force flag not set.
    #[error("destination already exists: {path}")]
    DestinationExists { path: PathBuf },

    /// Source is a directory but --whole-dir flag not set.
    #[error("source is a directory: {path}")]
    IsDirectory { path: PathBuf },

    /// Failed to move file.
    #[error("failed to move {src} to {dest}: {reason}")]
    MoveFailed {
        src: PathBuf,
        dest: PathBuf,
        reason: String,
    },

    /// Failed to copy file (cross-filesystem).
    #[error("failed to copy {src} to {dest}: {reason}")]
    CopyFailed {
        src: PathBuf,
        dest: PathBuf,
        reason: String,
    },

    /// File copied but failed to remove source.
    /// This is a warning state - file exists in both locations.
    #[error("copied but failed to remove source {src}: {reason}")]
    RemoveFailed {
        src: PathBuf,
        dest: PathBuf,
        reason: String,
    },

    /// Failed to create symlink.
    #[error("failed to create symlink {link} -> {target}: {reason}")]
    SymlinkFailed {
        link: PathBuf,
        target: PathBuf,
        reason: String,
    },

    /// Failed to create destination directory.
    #[error("failed to create directory {path}: {reason}")]
    CreateDirFailed { path: PathBuf, reason: String },

    /// I/O error wrapper.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for mvln operations.
pub type Result<T> = std::result::Result<T, MvlnError>;
