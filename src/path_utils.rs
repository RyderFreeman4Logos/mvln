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
        // For absolute mode, canonicalize or return as-is
        target_file
            .canonicalize()
            .unwrap_or_else(|_| target_file.to_path_buf())
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
        // In absolute mode, we try to canonicalize but fall back to original
        // Since /x/y/file doesn't exist in tests, it returns the path as-is
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
}
