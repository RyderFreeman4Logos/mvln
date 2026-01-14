//! Glob pattern expansion for mvln.
//!
//! This module provides functionality to expand glob patterns (like `*.txt` or `src/**/*.rs`)
//! into actual file paths, while also handling non-glob regular paths.
//!
//! # Examples
//!
//! ```
//! use mvln::glob_expand::{expand_globs, is_glob_pattern};
//!
//! // Check if a pattern contains glob metacharacters
//! assert!(is_glob_pattern("*.txt"));
//! assert!(is_glob_pattern("src/**/*.rs"));
//! assert!(!is_glob_pattern("regular_file.txt"));
//!
//! // Expand patterns to paths
//! let patterns = vec!["Cargo.toml".to_string()];
//! let paths = expand_globs(&patterns).unwrap();
//! assert!(!paths.is_empty());
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during glob expansion.
#[derive(Error, Debug)]
pub enum GlobError {
    /// The glob pattern syntax is invalid.
    #[error("invalid glob pattern '{pattern}': {source}")]
    InvalidPattern {
        pattern: String,
        source: glob::PatternError,
    },

    /// Glob expansion failed.
    #[error("failed to expand glob pattern '{pattern}': {source}")]
    ExpansionFailed {
        pattern: String,
        source: glob::GlobError,
    },

    /// No files matched the glob pattern.
    #[error("no files matched pattern: {pattern}")]
    NoMatches { pattern: String },
}

/// Check if a string contains glob metacharacters.
///
/// Returns `true` if the string contains any of: `*`, `?`, `[`, `]`
///
/// # Examples
///
/// ```
/// use mvln::glob_expand::is_glob_pattern;
///
/// assert!(is_glob_pattern("*.txt"));
/// assert!(is_glob_pattern("file?.log"));
/// assert!(is_glob_pattern("test[123].dat"));
/// assert!(!is_glob_pattern("regular_file.txt"));
/// assert!(!is_glob_pattern("/path/to/file"));
/// ```
#[must_use]
pub fn is_glob_pattern(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[') || s.contains(']')
}

/// Expand glob patterns to matching file paths.
///
/// If a pattern contains glob metacharacters (`*`, `?`, `[`, `]`), it will be expanded
/// to all matching paths. Otherwise, the path is returned as-is (even if it doesn't exist).
///
/// Results are sorted alphabetically for consistent output.
///
/// # Errors
///
/// Returns [`GlobError`] if:
/// - The glob pattern syntax is invalid
/// - Glob expansion fails due to I/O errors
/// - A glob pattern matches no files
///
/// # Examples
///
/// ```no_run
/// use mvln::glob_expand::expand_globs;
///
/// // Expand a glob pattern
/// let patterns = vec!["src/*.rs".to_string()];
/// let paths = expand_globs(&patterns)?;
/// // paths contains all .rs files in src/
///
/// // Mix glob patterns and regular paths
/// let patterns = vec![
///     "*.toml".to_string(),
///     "README.md".to_string(),
/// ];
/// let paths = expand_globs(&patterns)?;
/// # Ok::<(), mvln::glob_expand::GlobError>(())
/// ```
pub fn expand_globs(patterns: &[String]) -> Result<Vec<PathBuf>, GlobError> {
    let mut all_paths = Vec::new();

    for pattern in patterns {
        if is_glob_pattern(pattern) {
            // Expand glob pattern
            let glob_iter = glob::glob(pattern).map_err(|e| GlobError::InvalidPattern {
                pattern: pattern.clone(),
                source: e,
            })?;

            let mut matched_paths = Vec::new();
            for entry in glob_iter {
                let path = entry.map_err(|e| GlobError::ExpansionFailed {
                    pattern: pattern.clone(),
                    source: e,
                })?;
                matched_paths.push(path);
            }

            // Error if glob pattern matched nothing
            if matched_paths.is_empty() {
                return Err(GlobError::NoMatches {
                    pattern: pattern.clone(),
                });
            }

            all_paths.extend(matched_paths);
        } else {
            // Regular path, add as-is (even if it doesn't exist)
            // Existence check will be done by the caller
            all_paths.push(PathBuf::from(pattern));
        }
    }

    // Sort for consistent output
    all_paths.sort();

    Ok(all_paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_glob_pattern() {
        // Patterns with glob metacharacters
        assert!(is_glob_pattern("*.txt"));
        assert!(is_glob_pattern("file?.log"));
        assert!(is_glob_pattern("test[123].dat"));
        assert!(is_glob_pattern("src/**/*.rs"));
        assert!(is_glob_pattern("file[a-z].txt"));

        // Regular paths
        assert!(!is_glob_pattern("regular_file.txt"));
        assert!(!is_glob_pattern("/path/to/file"));
        assert!(!is_glob_pattern("dir/subdir/file.log"));
    }

    #[test]
    fn test_expand_single_regular_path() {
        let patterns = vec!["Cargo.toml".to_string()];
        let result = expand_globs(&patterns).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], PathBuf::from("Cargo.toml"));
    }

    #[test]
    fn test_expand_multiple_regular_paths() {
        let patterns = vec![
            "file1.txt".to_string(),
            "file2.txt".to_string(),
            "file3.txt".to_string(),
        ];
        let result = expand_globs(&patterns).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], PathBuf::from("file1.txt"));
        assert_eq!(result[1], PathBuf::from("file2.txt"));
        assert_eq!(result[2], PathBuf::from("file3.txt"));
    }

    #[test]
    fn test_expand_glob_cargo_toml() {
        // This test uses actual files in the project
        let patterns = vec!["Cargo.toml".to_string()];
        let result = expand_globs(&patterns).unwrap();
        assert!(!result.is_empty());
        assert!(result[0].to_str().unwrap().contains("Cargo.toml"));
    }

    #[test]
    fn test_expand_glob_with_wildcard() {
        // Test with actual Cargo.toml file
        let patterns = vec!["Cargo.*".to_string()];
        let result = expand_globs(&patterns).unwrap();
        assert!(!result.is_empty());
        assert!(result
            .iter()
            .any(|p| p.to_str().unwrap().contains("Cargo.toml")));
    }

    #[test]
    fn test_results_are_sorted() {
        // Mix regular paths to test sorting
        let patterns = vec![
            "zebra.txt".to_string(),
            "alpha.txt".to_string(),
            "beta.txt".to_string(),
        ];
        let result = expand_globs(&patterns).unwrap();
        assert_eq!(result[0], PathBuf::from("alpha.txt"));
        assert_eq!(result[1], PathBuf::from("beta.txt"));
        assert_eq!(result[2], PathBuf::from("zebra.txt"));
    }

    #[test]
    fn test_nonexistent_glob_returns_error() {
        let patterns = vec!["nonexistent_*.xyz".to_string()];
        let result = expand_globs(&patterns);
        assert!(result.is_err());
        match result {
            Err(GlobError::NoMatches { pattern }) => {
                assert_eq!(pattern, "nonexistent_*.xyz");
            }
            _ => panic!("Expected NoMatches error"),
        }
    }

    #[test]
    fn test_invalid_glob_pattern() {
        // Unclosed bracket is invalid glob syntax
        let patterns = vec!["file[abc".to_string()];
        let result = expand_globs(&patterns);
        assert!(result.is_err());
        assert!(matches!(result, Err(GlobError::InvalidPattern { .. })));
    }
}
