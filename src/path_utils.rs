//! Path utilities for symlink target computation.

use std::path::{Path, PathBuf};

/// Compute the symlink target path.
///
/// When creating a symlink at `link_location` pointing to `target_file`,
/// this function computes what the symlink content should be.
///
/// # Arguments
///
/// * `link_location` - Where the symlink will be created
/// * `target_file` - The actual file the symlink should point to
/// * `absolute` - If true, return absolute path; otherwise compute relative
///
/// # Examples
///
/// ```
/// use mvln::path_utils::compute_symlink_target;
///
/// // Relative path computation
/// let target = compute_symlink_target("/a/b/link", "/a/c/file", false);
/// assert_eq!(target.to_str().unwrap(), "../c/file");
///
/// // Absolute path
/// let target = compute_symlink_target("/a/b/link", "/a/c/file", true);
/// assert_eq!(target.to_str().unwrap(), "/a/c/file");
/// ```
pub fn compute_symlink_target<P: AsRef<Path>, Q: AsRef<Path>>(
    link_location: P,
    target_file: Q,
    absolute: bool,
) -> PathBuf {
    let target_file = target_file.as_ref();

    if absolute {
        // For absolute mode, ensure we return an absolute path
        // Try to canonicalize first (if file exists)
        if let Ok(canonicalized) = target_file.canonicalize() {
            return canonicalized;
        }

        // File doesn't exist, manually construct absolute path
        if target_file.is_absolute() {
            // Already absolute, use as-is
            target_file.to_path_buf()
        } else {
            // Relative path, convert to absolute based on current directory
            std::env::current_dir()
                .map_or_else(|_| target_file.to_path_buf(), |cwd| cwd.join(target_file))
        }
    } else {
        // Compute relative path from link location to target
        let link_location = link_location.as_ref();

        // Get the parent directory of the link (the symlink lives here)
        let link_dir = link_location.parent().unwrap_or(Path::new("."));

        // Use pathdiff to compute relative path
        pathdiff::diff_paths(target_file, link_dir).unwrap_or_else(|| target_file.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_path_returns_target_directly() {
        let result = compute_symlink_target("/a/b/link", "/x/y/file", true);
        // In absolute mode, we try to canonicalize first.
        // Since /x/y/file doesn't exist in tests, canonicalize fails
        // and we return the absolute path as-is.
        assert_eq!(result, PathBuf::from("/x/y/file"));
    }

    #[test]
    fn relative_path_same_directory() {
        // Link at /a/b/link pointing to /a/b/file -> just "file"
        let result = compute_symlink_target("/a/b/link", "/a/b/file", false);
        assert_eq!(result, PathBuf::from("file"));
    }

    #[test]
    fn relative_path_sibling_directory() {
        // Link at /a/b/link pointing to /a/c/file -> ../c/file
        let result = compute_symlink_target("/a/b/link", "/a/c/file", false);
        assert_eq!(result, PathBuf::from("../c/file"));
    }

    #[test]
    fn relative_path_different_branches() {
        // Link at /a/b/c/link pointing to /x/y/file -> ../../../x/y/file
        let result = compute_symlink_target("/a/b/c/link", "/x/y/file", false);
        assert_eq!(result, PathBuf::from("../../../x/y/file"));
    }

    #[test]
    fn absolute_mode_with_relative_target() {
        // When absolute=true and target is relative, convert to absolute
        let result = compute_symlink_target("/a/b/link", "relative/file.txt", true);
        // Result should be absolute (joined with current directory)
        assert!(
            result.is_absolute(),
            "Expected absolute path, got: {:?}",
            result
        );
    }

    #[test]
    fn absolute_mode_with_absolute_target() {
        // When absolute=true and target is already absolute, keep as-is
        let result = compute_symlink_target("/a/b/link", "/absolute/path/file.txt", true);
        assert_eq!(result, PathBuf::from("/absolute/path/file.txt"));
    }
}
