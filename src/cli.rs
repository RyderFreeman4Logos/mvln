//! Command-line interface definitions for mvls.
//!
//! This module provides the CLI structure and argument parsing using clap.
//! It handles validation of command-line arguments and converts them into
//! the internal `MoveOptions` type used by the core logic.

use clap::Parser;
use mvln::operation::MoveOptions;
use std::path::PathBuf;

/// Move files with flexible path resolution
///
/// mvls supports both relative and absolute path modes when moving files.
/// By default, it uses relative paths from the destination directory.
#[derive(Parser, Debug)]
#[command(name = "mvln")]
#[command(author, version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Source file(s) or directory to move
    ///
    /// Accepts one or more paths. If multiple sources are provided,
    /// the destination must be a directory.
    #[arg(required = true)]
    pub source: Vec<PathBuf>,

    /// Destination path (file or directory)
    ///
    /// If moving multiple sources, this must be a directory.
    #[arg(required = true)]
    pub dest: PathBuf,

    /// Use relative paths from the destination directory
    ///
    /// When creating symbolic links, paths will be relative to the
    /// destination directory. This is the default behavior.
    #[arg(short = 'r', long, conflicts_with = "absolute")]
    pub relative: bool,

    /// Use absolute paths for symbolic links
    ///
    /// When creating symbolic links, use absolute paths instead of
    /// relative paths.
    #[arg(short = 'a', long, conflicts_with = "relative")]
    pub absolute: bool,

    /// Move entire directory instead of just contents
    ///
    /// When the source is a directory, move the directory itself
    /// rather than its contents. This flag is CLI-specific and
    /// controls the behavior before core logic is invoked.
    #[arg(short = 'w', long)]
    pub whole_dir: bool,

    /// Enable verbose output
    ///
    /// Print detailed information about operations being performed.
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

impl Cli {
    /// Convert CLI arguments to `MoveOptions`
    ///
    /// This method translates the CLI representation into the core
    /// library's `MoveOptions` type. It handles the default behavior
    /// where neither -r nor -a is specified (defaults to relative).
    ///
    /// # Returns
    ///
    /// A `MoveOptions` instance ready for use by core move logic.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mvls::cli::Cli;
    /// use clap::Parser;
    ///
    /// let cli = Cli::parse();
    /// let options = cli.to_move_options();
    /// ```
    pub fn to_move_options(&self) -> MoveOptions {
        MoveOptions {
            absolute: self.absolute,
            force: false,   // CLI doesn't have force flag yet (future enhancement)
            dry_run: false, // Dry-run will be handled in main.rs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_to_relative() {
        let cli = Cli {
            source: vec![PathBuf::from("src")],
            dest: PathBuf::from("dst"),
            relative: false,
            absolute: false,
            whole_dir: false,
            verbose: false,
        };

        let options = cli.to_move_options();
        assert!(!options.absolute); // Default is relative
    }

    #[test]
    fn test_explicit_relative() {
        let cli = Cli {
            source: vec![PathBuf::from("src")],
            dest: PathBuf::from("dst"),
            relative: true,
            absolute: false,
            whole_dir: false,
            verbose: false,
        };

        let options = cli.to_move_options();
        assert!(!options.absolute); // Explicit relative
    }

    #[test]
    fn test_explicit_absolute() {
        let cli = Cli {
            source: vec![PathBuf::from("src")],
            dest: PathBuf::from("dst"),
            relative: false,
            absolute: true,
            whole_dir: false,
            verbose: false,
        };

        let options = cli.to_move_options();
        assert!(options.absolute); // Explicit absolute
    }

    #[test]
    fn test_multiple_sources() {
        let cli = Cli {
            source: vec![
                PathBuf::from("file1.txt"),
                PathBuf::from("file2.txt"),
                PathBuf::from("dir"),
            ],
            dest: PathBuf::from("target"),
            relative: false,
            absolute: false,
            whole_dir: false,
            verbose: false,
        };

        assert_eq!(cli.source.len(), 3);
    }
}
