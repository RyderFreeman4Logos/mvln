//! Core move-and-link operations.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::error::{MvlnError, Result};
use crate::path_utils::compute_symlink_target;

/// Options for `move_and_link` operation.
#[derive(Debug, Clone, Default)]
pub struct MoveOptions {
    /// Use absolute paths for symlinks instead of relative.
    pub absolute: bool,
    /// Overwrite existing destination.
    pub force: bool,
    /// Only print commands, don't execute.
    pub dry_run: bool,
}

/// Result of a successful `move_and_link` operation.
#[derive(Debug)]
pub struct MoveResult {
    /// The source path (now a symlink).
    pub source: PathBuf,
    /// The destination path (where file was moved).
    pub dest: PathBuf,
    /// The symlink target (what the symlink points to).
    pub symlink_target: PathBuf,
}

/// Move a file to destination and create a symlink at the original location.
///
/// # Safety Guarantees
///
/// - The file is NEVER lost. If symlink creation fails, the file remains
///   at the destination and an error is returned with recovery instructions.
/// - For cross-filesystem moves, the file is fully copied and verified
///   before the source is removed.
///
/// # Arguments
///
/// * `source` - The source file or directory to move
/// * `dest` - The destination path
/// * `options` - Operation options
///
/// # Errors
///
/// Returns an error if:
/// - Source does not exist
/// - Destination exists and force is not set
/// - Move operation fails
/// - Symlink creation fails (file is preserved at destination)
pub fn move_and_link<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    dest: Q,
    options: &MoveOptions,
) -> Result<MoveResult> {
    let source = source.as_ref();
    let dest = dest.as_ref();

    // Step 1: Verify source exists (including dangling symlinks)
    // Use symlink_metadata instead of exists() to detect dangling symlinks
    // Also distinguish between "not found" and other I/O errors (permission denied, etc.)
    match source.symlink_metadata() {
        Ok(_) => {} // Source exists
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Err(MvlnError::SourceNotFound {
                path: source.to_path_buf(),
            });
        }
        Err(e) => {
            return Err(MvlnError::SourceAccessError {
                path: source.to_path_buf(),
                reason: e.to_string(),
            });
        }
    }

    // Step 2: Resolve destination path
    // If dest is a directory, append source filename
    let dest = resolve_destination(source, dest);

    // Step 2.5: Check source != dest (prevent self-move data loss)
    // Use absolute_path_no_follow to handle symlinks correctly - don't follow them.
    let source_canonical = absolute_path_no_follow(source);
    let dest_canonical = absolute_path_no_follow(&dest);

    if source_canonical == dest_canonical {
        return Err(MvlnError::SameSourceAndDest {
            path: source.to_path_buf(),
        });
    }

    // Step 2.6: Check dest is not inside source (prevent infinite recursion)
    // This can happen when moving a directory to its own subdirectory,
    // e.g., `mvln dir dir/subdir` would cause copy_dir_recursive to loop forever.
    // Only check for actual directories (not symlinks to directories).
    let source_is_symlink = source
        .symlink_metadata()
        .map(|m| m.is_symlink())
        .unwrap_or(false);
    let source_is_real_dir = !source_is_symlink && source.is_dir();
    if source_is_real_dir && dest_canonical.starts_with(&source_canonical) {
        return Err(MvlnError::DestinationInsideSource {
            src: source.to_path_buf(),
            dest: dest.clone(),
        });
    }

    // Step 3: Check destination doesn't exist (unless force)
    // Use symlink_metadata to detect dangling symlinks at destination
    let dest_exists = dest.symlink_metadata().is_ok();
    if dest_exists && !options.force {
        return Err(MvlnError::DestinationExists { path: dest.clone() });
    }

    // Step 4: Compute symlink target
    let symlink_target = compute_symlink_target(source, &dest, options.absolute);

    // Step 5: Dry-run mode - return without making changes
    if options.dry_run {
        return Ok(MoveResult {
            source: source.to_path_buf(),
            dest,
            symlink_target,
        });
    }

    // Step 6: Create destination parent directories
    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| MvlnError::CreateDirFailed {
                path: parent.to_path_buf(),
                reason: e.to_string(),
            })?;
        }
    }

    // Step 7: Remove destination if force and exists
    // SAFETY: Check symlink FIRST to avoid following symlinks to directories.
    // is_dir() follows symlinks, so a symlink->dir would cause remove_dir_all
    // to delete the target directory contents instead of just the symlink.
    if dest_exists && options.force {
        if dest.is_symlink() {
            // Remove symlink itself, not the target
            fs::remove_file(&dest).map_err(|e| MvlnError::MoveFailed {
                src: source.to_path_buf(),
                dest: dest.clone(),
                reason: format!("failed to remove existing symlink: {e}"),
            })?;
        } else if dest.is_dir() {
            // Actual directory (not symlink), safe to remove recursively
            fs::remove_dir_all(&dest).map_err(|e| MvlnError::MoveFailed {
                src: source.to_path_buf(),
                dest: dest.clone(),
                reason: format!("failed to remove existing directory: {e}"),
            })?;
        } else {
            // Regular file
            fs::remove_file(&dest).map_err(|e| MvlnError::MoveFailed {
                src: source.to_path_buf(),
                dest: dest.clone(),
                reason: format!("failed to remove existing file: {e}"),
            })?;
        }
    }

    // Step 8: Move the file/directory
    move_file(source, &dest)?;

    // Step 9: Create symlink at original location
    create_symlink(source, &dest, &symlink_target)?;

    Ok(MoveResult {
        source: source.to_path_buf(),
        dest,
        symlink_target,
    })
}

/// Resolve destination path: if dest is directory, append source filename.
fn resolve_destination(source: &Path, dest: &Path) -> PathBuf {
    if dest.is_dir() {
        if let Some(filename) = source.file_name() {
            return dest.join(filename);
        }
    }
    dest.to_path_buf()
}

/// Compute absolute path for a path without following symlinks.
/// If the path is a symlink, canonicalize the parent and join with filename.
/// If the path doesn't exist, build absolute path from parent.
fn absolute_path_no_follow(path: &Path) -> PathBuf {
    let is_symlink = path
        .symlink_metadata()
        .map(|m| m.is_symlink())
        .unwrap_or(false);

    if is_symlink {
        // For symlinks, canonicalize parent and join with filename
        std::fs::canonicalize(path.parent().unwrap_or(Path::new("."))).map_or_else(
            |_| path.to_path_buf(),
            |p| p.join(path.file_name().unwrap_or_default()),
        )
    } else if let Ok(canonical) = path.canonicalize() {
        canonical
    } else {
        // Path doesn't exist - build absolute path from parent
        path.parent()
            .map(|p| {
                if p.as_os_str().is_empty() {
                    Path::new(".")
                } else {
                    p
                }
            })
            .and_then(|p| p.canonicalize().ok())
            .map_or_else(
                || path.to_path_buf(),
                |p| p.join(path.file_name().unwrap_or_default()),
            )
    }
}

/// Move file or directory from source to dest.
/// Uses rename for same filesystem, falls back to copy+remove for cross-filesystem.
fn move_file(source: &Path, dest: &Path) -> Result<()> {
    // Try atomic rename first
    match fs::rename(source, dest) {
        Ok(()) => Ok(()),
        Err(e) if is_cross_device_error(&e) => {
            // Cross-filesystem: copy then remove
            copy_and_remove(source, dest)
        }
        Err(e) => Err(MvlnError::MoveFailed {
            src: source.to_path_buf(),
            dest: dest.to_path_buf(),
            reason: e.to_string(),
        }),
    }
}

/// Check if error is cross-device link error (EXDEV).
fn is_cross_device_error(e: &std::io::Error) -> bool {
    // On Unix, cross-device move returns EXDEV (errno 18)
    // std::io::Error doesn't have a specific variant, so we check raw_os_error
    e.raw_os_error() == Some(libc::EXDEV)
}

/// Copy source to dest, verify, then remove source.
fn copy_and_remove(source: &Path, dest: &Path) -> Result<()> {
    // SAFETY: Check symlink FIRST before checking is_dir().
    // is_dir() follows symlinks, which could lead to:
    // 1. Copying target contents instead of the symlink itself
    // 2. Traversing outside the source tree
    // 3. remove_dir_all following the symlink and deleting target contents
    if source.is_symlink() {
        // Copy the symlink itself, not its target
        let target = fs::read_link(source).map_err(|e| MvlnError::CopyFailed {
            src: source.to_path_buf(),
            dest: dest.to_path_buf(),
            reason: format!("failed to read symlink: {e}"),
        })?;

        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, dest).map_err(|e| MvlnError::CopyFailed {
            src: source.to_path_buf(),
            dest: dest.to_path_buf(),
            reason: format!("failed to create symlink: {e}"),
        })?;

        #[cfg(not(unix))]
        {
            return Err(MvlnError::CopyFailed {
                src: source.to_path_buf(),
                dest: dest.to_path_buf(),
                reason: "symlinks not supported on this platform".to_string(),
            });
        }

        // Remove the original symlink (not its target)
        fs::remove_file(source).map_err(|e| MvlnError::RemoveFailed {
            src: source.to_path_buf(),
            dest: dest.to_path_buf(),
            reason: format!("failed to remove symlink: {e}"),
        })?;

        return Ok(());
    }

    // Not a symlink - proceed with regular file/directory copy
    if source.is_dir() {
        copy_dir_recursive(source, dest)?;
    } else {
        fs::copy(source, dest).map_err(|e| MvlnError::CopyFailed {
            src: source.to_path_buf(),
            dest: dest.to_path_buf(),
            reason: e.to_string(),
        })?;

        // Attempt to preserve modification time
        if let Ok(metadata) = source.metadata() {
            if let Ok(mtime) = metadata.modified() {
                if let Ok(dest_file) = fs::File::open(dest) {
                    let _ = dest_file.set_modified(mtime);
                }
            }
        }
    }

    // Verify copy succeeded before removing source
    if !dest.exists() {
        return Err(MvlnError::CopyFailed {
            src: source.to_path_buf(),
            dest: dest.to_path_buf(),
            reason: "destination not found after copy".to_string(),
        });
    }

    // Remove source
    let remove_result = if source.is_dir() {
        fs::remove_dir_all(source)
    } else {
        fs::remove_file(source)
    };

    if let Err(e) = remove_result {
        return Err(MvlnError::RemoveFailed {
            src: source.to_path_buf(),
            dest: dest.to_path_buf(),
            reason: e.to_string(),
        });
    }

    Ok(())
}

/// Recursively copy a directory.
fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).map_err(|e| MvlnError::CreateDirFailed {
        path: dest.to_path_buf(),
        reason: e.to_string(),
    })?;

    for entry in fs::read_dir(source).map_err(|e| MvlnError::CopyFailed {
        src: source.to_path_buf(),
        dest: dest.to_path_buf(),
        reason: e.to_string(),
    })? {
        let entry = entry.map_err(|e| MvlnError::CopyFailed {
            src: source.to_path_buf(),
            dest: dest.to_path_buf(),
            reason: e.to_string(),
        })?;

        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        // SAFETY: Check symlink FIRST before is_dir().
        // is_dir() follows symlinks, which could cause:
        // 1. Recursing into directories outside the source tree
        // 2. Copying target contents instead of the symlink itself
        if src_path.is_symlink() {
            // Copy the symlink itself, not its target
            let target = fs::read_link(&src_path).map_err(|e| MvlnError::CopyFailed {
                src: src_path.clone(),
                dest: dest_path.clone(),
                reason: format!("failed to read symlink: {e}"),
            })?;

            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &dest_path).map_err(|e| MvlnError::CopyFailed {
                src: src_path.clone(),
                dest: dest_path.clone(),
                reason: format!("failed to create symlink: {e}"),
            })?;

            #[cfg(not(unix))]
            {
                return Err(MvlnError::CopyFailed {
                    src: src_path.clone(),
                    dest: dest_path,
                    reason: "symlinks not supported on this platform".to_string(),
                });
            }

            // Continue to next entry - do NOT recurse into the symlink
            continue;
        }

        // Not a symlink - check if directory or regular file
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path).map_err(|e| MvlnError::CopyFailed {
                src: src_path.clone(),
                dest: dest_path.clone(),
                reason: e.to_string(),
            })?;

            // Attempt to preserve modification time
            if let Ok(metadata) = src_path.metadata() {
                if let Ok(mtime) = metadata.modified() {
                    if let Ok(dest_file) = fs::File::open(&dest_path) {
                        let _ = dest_file.set_modified(mtime);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Create symlink at source location pointing to destination.
fn create_symlink(source: &Path, dest: &Path, symlink_target: &Path) -> Result<()> {
    // Remove any existing file/symlink at source location
    // (source was moved, so it shouldn't exist, but handle edge cases)
    if source.exists() || source.is_symlink() {
        match fs::remove_file(source) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => {
                return Err(MvlnError::SymlinkFailed {
                    link: source.to_path_buf(),
                    target: symlink_target.to_path_buf(),
                    reason: format!("failed to remove existing file at source: {e}"),
                });
            }
        }
    }

    // Create symlink
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(symlink_target, source).map_err(|e| {
            MvlnError::SymlinkFailed {
                link: source.to_path_buf(),
                target: dest.to_path_buf(),
                reason: e.to_string(),
            }
        })?;
    }

    #[cfg(not(unix))]
    {
        return Err(MvlnError::SymlinkFailed {
            link: source.to_path_buf(),
            target: dest.to_path_buf(),
            reason: "symlinks not supported on this platform".to_string(),
        });
    }

    Ok(())
}
