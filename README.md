# mvln

Move files and create symlinks at their original locations.

`mvln` is a command-line utility that moves files or directories to a new location while preserving access at the original path through symbolic links. This is particularly useful when reorganizing files while maintaining compatibility with tools or scripts that expect files at their original locations.

## Features

- **Transparent File Movement**: Move files while preserving access through symlinks
- **Glob Pattern Support**: Move multiple files matching patterns (e.g., `*.txt`)
- **Directory Handling**: Move entire directories with the `-w/--whole-dir` flag
- **Flexible Symlink Modes**: Create relative (default) or absolute symlinks
- **Force Overwrite**: Replace existing destinations with `-f/--force`
- **Internationalization**: Supports multiple languages based on system locale
- **Safe Operations**: Type-checked operations prevent unsafe cross-type replacements
- **Verbose Mode**: Detailed operation logging with `-v/--verbose`

## Installation

```bash
git clone https://github.com/RyderFreeman4Logos/mvls.git
cd mvls
just install
```

## Usage

### Basic Syntax

```bash
mvln [OPTIONS] <SOURCE>... <DESTINATION>
```

### Examples

#### Move a Single File

```bash
# Move file.txt to /backup/, create symlink at original location
mvln file.txt /backup/

# Result:
# - /backup/file.txt (actual file)
# - file.txt -> ../backup/file.txt (symlink)
```

#### Move Multiple Files

```bash
# Move all .log files to archive/
mvln *.log archive/

# Each .log file is replaced with a symlink pointing to archive/
```

#### Move a Directory

```bash
# Move entire directory (requires -w flag)
mvln -w ./old_project /archive/

# Result:
# - /archive/old_project/ (moved directory)
# - ./old_project -> ../archive/old_project (symlink)
```

#### Use Absolute Symlinks

```bash
# Create absolute symlinks instead of relative
mvln -a config.toml /etc/myapp/

# Result:
# - /etc/myapp/config.toml (actual file)
# - config.toml -> /etc/myapp/config.toml (absolute symlink)
```

#### Force Overwrite

```bash
# Replace existing destination file (same type only)
mvln -f new_version.bin /usr/local/bin/tool.bin

# Replaces existing tool.bin with new_version.bin
```

#### Verbose Output

```bash
# Show detailed operation information
mvln -v data.db /mnt/storage/

# Output:
# mv data.db /mnt/storage/
# ln -s ../mnt/storage/data.db data.db
# Moving: data.db -> /mnt/storage/data.db
# Creating symlink: data.db -> ../mnt/storage/data.db
# Completed: 1 file(s) moved, 1 symlink(s) created
```

## Command-Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--relative` | `-r` | Create relative symlinks (default behavior) |
| `--absolute` | `-a` | Create absolute symlinks instead of relative |
| `--whole-dir` | `-w` | Move entire directory instead of contents |
| `--verbose` | `-v` | Enable verbose output |
| `--force` | `-f` | Overwrite existing destination (same type only) |
| `--help` | `-h` | Display help information |
| `--version` | `-V` | Display version information |

## Behavior Details

### Symlink Path Resolution

**Default (Relative Mode)**:
- Symlinks use relative paths from the original location
- Portable across different mount points
- Example: `file.txt -> ../backup/file.txt`

**Absolute Mode (`-a`):**
- Symlinks use absolute paths
- More robust when moving symlinks themselves
- Example: `file.txt -> /home/user/backup/file.txt`

### Directory Handling

By default, `mvln` rejects directory sources to prevent accidental moves. Use the `-w/--whole-dir` flag to explicitly move directories:

```bash
# Without -w: Error
mvln my_dir /backup/
# Error: my_dir is a directory, use -w/--whole-dir flag

# With -w: Success
mvln -w my_dir /backup/
# my_dir is moved to /backup/my_dir, symlink created
```

### Force Overwrite Rules

The `-f/--force` flag allows overwriting existing destinations with the following constraints:

- **File → File**: Allowed (replaces destination file)
- **Directory → Directory**: Allowed (merges into destination directory)
- **File → Directory**: Moves file *into* directory (standard behavior)
- **Directory → File**: **Rejected** (type mismatch)

### Glob Pattern Expansion

`mvln` natively supports glob patterns:

```bash
# Move all .txt files
mvln *.txt archive/

# Move files matching pattern
mvln report_202*.pdf /backup/reports/

# Multiple patterns
mvln *.log *.txt /archive/
```

### Error Recovery

If symlink creation fails after moving a file, `mvln` provides a recovery command:

```
Error: Failed to create symlink at file.txt

File was successfully moved to /backup/file.txt
To recover the original state, run:
  mv /backup/file.txt file.txt
```

## Internationalization

`mvln` supports multiple languages based on your system locale:

- English (en)
- Chinese Simplified (zh-CN)
- (More languages can be added via Fluent translation files)

Messages, error descriptions, and hints are automatically localized.

## Platform Support

- **Unix/Linux**: Full support
- **macOS**: Full support
- **Windows**: Limited (symbolic link creation may require administrator privileges)

## Safety Guarantees

- **Type Safety**: Prevents replacing files with directories and vice versa (unless moving into a directory)
- **No Unsafe Code**: The codebase forbids `unsafe` blocks
- **Atomic Operations**: File moves use filesystem primitives for atomicity
- **Symlink Validation**: Verifies symlink creation and target resolution

## Use Cases

### Reorganizing Large Codebases

```bash
# Move source files to organized directory structure
mvln -w src/ archive/2024-01-backup/
# src/ is now a symlink, builds still work
```

### Log Rotation

```bash
# Move logs to archive while preserving access
mvln /var/log/app.log /archive/logs/
# /var/log/app.log -> /archive/logs/app.log
```

### Configuration Management

```bash
# Move config to system location, keep local access
mvln -a config.toml /etc/myapp/
# config.toml -> /etc/myapp/config.toml (absolute)
```

## Building from Source

### Prerequisites

- Rust 2021 edition or later
- [just](https://github.com/casey/just) command runner

### Build Commands

```bash
# Debug build
just build

# Release build
just build-release

# Run all tests
just test

# Run linter (Clippy)
just clippy

# Run all pre-commit checks (format, clippy, test)
just pre-commit

# Install locally
just install
```

## Development

### Project Structure

```
mvln/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── cli.rs           # Argument parsing
│   ├── lib.rs           # Library exports
│   ├── operation.rs     # Core move-and-link logic
│   ├── path_utils.rs    # Path computation utilities
│   ├── glob_expand.rs   # Glob pattern expansion
│   ├── error.rs         # Error types
│   └── i18n.rs          # Internationalization
├── tests/
│   ├── integration.rs   # Integration tests
│   └── safety.rs        # Safety tests
└── Cargo.toml
```

### Running Tests

```bash
# All tests
just test

# Integration tests only
just test-integration

# With output
just test-verbose
```

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Run pre-commit checks (`just pre-commit`)
4. Commit with clear messages (Conventional Commits format)
5. Push to your fork and submit a pull request

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- CLI parsing: [clap](https://github.com/clap-rs/clap)
- Internationalization: [Fluent](https://projectfluent.org/)
- Glob matching: [glob](https://github.com/rust-lang/glob)

## Links

- **Repository**: [https://github.com/RyderFreeman4Logos/mvls](https://github.com/RyderFreeman4Logos/mvls)
- **Issue Tracker**: [https://github.com/RyderFreeman4Logos/mvls/issues](https://github.com/RyderFreeman4Logos/mvls/issues)
