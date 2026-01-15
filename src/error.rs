//! Error types for mvln operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during mvln operations.
#[derive(Error, Debug)]
pub enum MvlnError {
    /// Source file or directory not found.
    #[error("source not found: {path}")]
    SourceNotFound { path: PathBuf },

    /// Cannot access source file or directory (permission denied, etc.).
    #[error("cannot access source {path}: {reason}")]
    SourceAccessError { path: PathBuf, reason: String },

    /// Destination already exists and force flag not set.
    #[error("destination already exists: {path}")]
    DestinationExists { path: PathBuf },

    /// Source is a directory but --whole-dir flag not set.
    #[error("source is a directory: {path}")]
    IsDirectory { path: PathBuf },

    /// Source and destination are the same path.
    #[error("source and destination are the same: {path}")]
    SameSourceAndDest { path: PathBuf },

    /// Destination is inside source directory (would cause infinite recursion).
    #[error("cannot move directory into itself: {src} -> {dest}")]
    DestinationInsideSource { src: PathBuf, dest: PathBuf },

    /// Type mismatch: cannot replace directory with file or vice versa.
    #[error("type mismatch: cannot replace {dest_type} with {src_type}: {src} -> {dest}")]
    TypeMismatch {
        src: PathBuf,
        dest: PathBuf,
        src_type: &'static str,
        dest_type: &'static str,
    },

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

    /// Invalid destination path.
    #[error("invalid destination: {reason}")]
    InvalidDestination { reason: String },

    /// Invalid source path.
    #[error("invalid path {path}: {reason}")]
    InvalidPath { path: PathBuf, reason: String },

    /// Glob expansion failed.
    #[error("glob expansion failed: {reason}")]
    GlobExpansionFailed { reason: String },

    /// Batch operation failed with multiple errors.
    #[error("{count} operation(s) failed")]
    BatchOperationFailed { count: usize },

    /// I/O error wrapper.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for mvln operations.
pub type Result<T> = std::result::Result<T, MvlnError>;
