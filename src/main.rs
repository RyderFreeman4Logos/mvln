//! mvln - Move files and create symlinks at original locations.
//!
//! This binary provides a command-line interface to the mvln library,
//! allowing users to move files while preserving access through symlinks.

use clap::Parser;
use fluent::FluentArgs;
use mvln::error::{MvlnError, Result};
use mvln::glob_expand::expand_globs;
use mvln::i18n;
use mvln::operation::move_and_link;
use std::path::{Path, PathBuf};
use std::process;

mod cli;
use cli::Cli;

/// Shell-escape a string by wrapping it in single quotes and escaping embedded quotes.
///
/// This ensures paths with spaces or special characters can be safely copied to a shell.
fn shell_escape(s: &str) -> String {
    // If string contains no special chars, return as-is
    if !s.contains(|c: char| {
        c.is_whitespace() || matches!(c, '\'' | '"' | '\\' | '$' | '`' | '!' | '*' | '?' | '[')
    }) {
        return s.to_string();
    }

    // Otherwise, wrap in single quotes and escape embedded single quotes
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// Print equivalent shell command for mv operation.
///
/// # Arguments
///
/// * `src_display` - Source path as entered by user (preserved for display)
/// * `dest_display` - Destination path as entered by user (preserved for display)
fn print_mv_command(src_display: &str, dest_display: &str) {
    println!(
        "mv {} {}",
        shell_escape(src_display),
        shell_escape(dest_display)
    );
}

/// Print equivalent shell command for ln -s operation.
///
/// # Arguments
///
/// * `target` - The symlink target (relative or absolute based on options)
/// * `link` - The symlink location
fn print_ln_command(target: &Path, link: &Path) {
    println!(
        "ln -s {} {}",
        shell_escape(&target.display().to_string()),
        shell_escape(&link.display().to_string())
    );
}

/// Print recovery command when symlink creation fails.
///
/// # Arguments
///
/// * `bundle` - Fluent bundle for i18n messages
/// * `dest` - Where the file was moved to
/// * `src` - Original source location
fn print_recovery_command(
    bundle: &fluent::FluentBundle<fluent::FluentResource>,
    dest: &Path,
    src: &Path,
) {
    let mut args = FluentArgs::new();
    args.set("dest", dest.display().to_string());
    println!("\n{}", i18n::msg(bundle, "recovery-header", Some(&args)));
    println!("{}", i18n::simple_msg(bundle, "recovery-command"));

    // Use shell-escaped paths for the command
    println!(
        "  mv {} {}",
        shell_escape(&dest.display().to_string()),
        shell_escape(&src.display().to_string())
    );
}

/// Main entry point for mvln CLI.
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}

/// Core application logic.
fn run() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize i18n
    let bundle = i18n::init();

    // Convert CLI arguments to library options
    let options = cli.to_move_options();

    // Expand glob patterns in source paths
    let source_paths = expand_sources(&cli.source)?;

    // Validate: if multiple sources, destination must be a directory
    if source_paths.len() > 1 && !cli.dest.is_dir() {
        return Err(MvlnError::InvalidDestination {
            reason: "destination must be a directory when moving multiple files".to_string(),
        });
    }

    // Track statistics
    let mut files_moved = 0;
    let mut symlinks_created = 0;
    let mut errors = Vec::new();

    // Process each source file
    for source in &source_paths {
        // Check if source is a directory (don't follow symlinks)
        let is_dir = source
            .symlink_metadata()
            .map(|m| m.is_dir())
            .unwrap_or(false);

        if is_dir && !cli.whole_dir {
            // Error: directory requires -w flag
            let mut args = FluentArgs::new();
            args.set("path", source.display().to_string());
            eprintln!("{}", i18n::msg(&bundle, "err-is-directory", Some(&args)));

            // Print hint about using -w or glob
            if let Some(attr) = bundle
                .get_message("err-is-directory")
                .and_then(|m| m.get_attribute("hint"))
            {
                let mut errors = vec![];
                let hint = bundle.format_pattern(attr.value(), Some(&args), &mut errors);
                eprintln!("  {hint}");
            }

            errors.push(MvlnError::InvalidPath {
                path: source.clone(),
                reason: "is a directory, use -w/--whole-dir flag".to_string(),
            });
            continue; // Skip this source
        }
        // Preserve user input format for display (important for mv command output)
        let src_display = find_original_input(&cli.source, source);

        // Print equivalent mv command (using user's original dest for display)
        print_mv_command(&src_display, &cli.dest.display().to_string());

        // Execute move-and-link operation
        // Note: move_and_link handles destination resolution (appending filename if dest is dir)
        match move_and_link(source, &cli.dest, &options) {
            Ok(result) => {
                // Print equivalent ln -s command
                print_ln_command(&result.symlink_target, &result.source);

                files_moved += 1;
                symlinks_created += 1;

                if cli.verbose {
                    let mut args = FluentArgs::new();
                    args.set("src", result.source.display().to_string());
                    args.set("dest", result.dest.display().to_string());
                    println!("{}", i18n::msg(&bundle, "op-moving", Some(&args)));

                    let mut link_args = FluentArgs::new();
                    link_args.set("link", result.source.display().to_string());
                    link_args.set("target", result.symlink_target.display().to_string());
                    println!("{}", i18n::msg(&bundle, "op-linking", Some(&link_args)));
                }
            }
            Err(e) => {
                // Handle symlink failure specially (file is preserved)
                if let MvlnError::SymlinkFailed { target, .. } = &e {
                    eprintln!("\n{e}");
                    print_recovery_command(&bundle, target, source);
                    files_moved += 1; // File was moved successfully
                } else {
                    eprintln!("\n{e}");
                }
                errors.push(e);
            }
        }
    }

    // Print completion summary
    println!();
    let mut summary_args = FluentArgs::new();
    summary_args.set("files", files_moved);
    summary_args.set("links", symlinks_created);
    println!("{}", i18n::msg(&bundle, "op-complete", Some(&summary_args)));

    // Return error if any operation failed
    if errors.is_empty() {
        Ok(())
    } else {
        Err(MvlnError::BatchOperationFailed {
            count: errors.len(),
        })
    }
}

/// Expand glob patterns in source arguments.
///
/// Regular paths are passed through as-is (existence check happens in `move_and_link`).
fn expand_sources(sources: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let patterns: Vec<String> = sources.iter().map(|p| p.display().to_string()).collect();

    expand_globs(&patterns).map_err(|e| MvlnError::GlobExpansionFailed {
        reason: e.to_string(),
    })
}

/// Find the original user input that corresponds to an expanded path.
///
/// This is used to preserve the user's input format in mv command output.
/// For example, if user typed `./file.txt`, we should print `mv ./file.txt ...`
/// not `mv file.txt ...`.
fn find_original_input(original_args: &[PathBuf], expanded_path: &Path) -> String {
    for arg in original_args {
        let arg_str = arg.display().to_string();

        // Exact match
        if arg == expanded_path {
            return arg_str;
        }

        // Normalized match: handle ./file vs file, trailing slashes, etc.
        // Compare by converting both to canonical form for comparison.
        let arg_normalized = arg.components().collect::<std::path::PathBuf>();
        let expanded_normalized = expanded_path.components().collect::<std::path::PathBuf>();
        if arg_normalized == expanded_normalized {
            return arg_str;
        }

        // If arg is a glob pattern that could have expanded to this path
        if mvln::glob_expand::is_glob_pattern(&arg_str) {
            // Return the expanded path display
            return expanded_path.display().to_string();
        }
    }

    // Fallback: return the expanded path
    expanded_path.display().to_string()
}
