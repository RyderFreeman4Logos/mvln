# mvln - English (US) translations

# Operation messages
op-moving = Moving { $src } -> { $dest }
op-linking = Creating symlink { $link } -> { $target }
op-complete = Complete: { $files } file(s) moved, { $links } symlink(s) created
op-dry-run = [DRY-RUN] No changes made

# Equivalent commands (debug output)
cmd-mv = mv { $src } { $dest }
cmd-ln = ln -s { $target } { $link }

# Error messages
err-source-not-found = Error: Source not found: { $path }
err-dest-exists = Error: Destination already exists: { $path }
    .hint = Use -f/--force to overwrite
err-is-directory = Error: { $path } is a directory
    .hint = Use -d/--whole-dir to move directories, or use glob pattern (e.g., { $path }/*)
err-symlink-failed = Error: Failed to create symlink { $link } -> { $target }
    .reason = Reason: { $reason }
err-move-failed = Error: Failed to move { $src } -> { $dest }
    .reason = Reason: { $reason }
err-copy-failed = Error: Failed to copy { $src } -> { $dest }
    .reason = Reason: { $reason }
err-remove-failed = Warning: File copied but failed to remove source: { $src }
    .reason = Reason: { $reason }
    .note = File exists in both locations. Manual cleanup may be needed.

# Recovery messages
recovery-header = File has been moved to: { $dest }
recovery-command = Recovery command (to rollback):
recovery-mv = mv { $dest } { $src }

# Help text
help-source = Source file(s) or glob pattern
help-dest = Destination path
help-force = Overwrite existing destination
help-whole-dir = Move directory as a unit (default: error on directory)
help-relative = Use relative symlinks (default)
help-absolute = Use absolute symlinks
help-dry-run = Print commands without executing
help-verbose = Verbose output
