//! mvln - Move files and create symlinks at original locations.
//!
//! This library provides the core functionality for moving files
//! while preserving access through symlinks.

pub mod error;
pub mod glob_expand;
pub mod i18n;
pub mod operation;
pub mod path_utils;

pub use error::{MvlnError, Result};
pub use glob_expand::{expand_globs, is_glob_pattern, GlobError};
pub use operation::{move_and_link, MoveOptions};
pub use path_utils::compute_symlink_target;
