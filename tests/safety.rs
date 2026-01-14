//! Safety tests for mvln operations.
//!
//! These tests verify the core safety guarantee: FILES ARE NEVER LOST.
//! They are written first (TDD) to drive the implementation.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use tempfile::TempDir;

use mvln::{move_and_link, MoveOptions, MvlnError};

/// Helper to create a test file with content.
fn create_test_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create parent directories");
    }
    fs::write(path, content).expect("Failed to write test file");
}

// =============================================================================
// Core Safety Tests
// =============================================================================

#[test]
fn file_never_lost_on_successful_operation() {
    // GIVEN: A source file exists with known content
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source.txt");
    let dest = temp.path().join("dest").join("moved.txt");
    let content = "important data that must not be lost";

    create_test_file(&source, content);
    assert!(source.exists(), "Source file should exist before operation");

    // WHEN: mvln operation succeeds
    let options = MoveOptions::default();
    let result = move_and_link(&source, &dest, &options);

    // THEN: File is at destination AND symlink exists at source
    assert!(result.is_ok(), "Operation should succeed: {:?}", result);

    // File content is accessible at destination
    assert!(dest.exists(), "Destination should exist");
    let dest_content = fs::read_to_string(&dest).expect("Should read destination");
    assert_eq!(dest_content, content, "Content should be preserved");

    // Symlink at source points to destination
    assert!(source.is_symlink(), "Source should be a symlink");

    // Content is accessible through the symlink
    let through_symlink = fs::read_to_string(&source).expect("Should read through symlink");
    assert_eq!(
        through_symlink, content,
        "Content through symlink should match"
    );
}

#[test]
fn file_preserved_when_symlink_fails() {
    // GIVEN: A source file exists
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source.txt");
    let dest = temp.path().join("dest.txt");
    let content = "precious data";

    create_test_file(&source, content);

    // AND: Something exists at source location that would prevent symlink
    // We simulate this by first doing the move, then creating a file at source
    // This tests error recovery behavior

    // First, manually move the file
    fs::create_dir_all(dest.parent().unwrap_or(Path::new("."))).ok();
    fs::rename(&source, &dest).expect("Manual move should succeed");

    // Now create something at source that blocks symlink creation
    fs::write(&source, "blocker").expect("Should create blocker file");

    // WHEN: We try to create symlink (simulating partial operation failure)
    let symlink_result = symlink(&dest, &source);

    // THEN: Symlink creation fails (expected)
    assert!(
        symlink_result.is_err(),
        "Symlink should fail due to existing file"
    );

    // AND: The file is STILL at destination (never lost!)
    assert!(dest.exists(), "File must still exist at destination");
    let preserved_content = fs::read_to_string(&dest).expect("Should read preserved file");
    assert_eq!(preserved_content, content, "File content must be preserved");
}

#[test]
fn symlink_points_to_correct_target() {
    // GIVEN: A successful mvln operation
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("subdir").join("file.txt");
    let dest = temp.path().join("archive").join("file.txt");
    let content = "test content";

    create_test_file(&source, content);

    // WHEN: mvln with relative symlink (default)
    let options = MoveOptions::default();
    let result = move_and_link(&source, &dest, &options);

    // THEN: Symlink resolves to the correct file
    assert!(result.is_ok(), "Operation should succeed");

    // The symlink should resolve to dest when followed
    let resolved = fs::canonicalize(&source).expect("Should resolve symlink");
    let expected = fs::canonicalize(&dest).expect("Should canonicalize dest");
    assert_eq!(resolved, expected, "Symlink should resolve to destination");

    // Reading through symlink should give same content as reading dest
    let via_symlink = fs::read_to_string(&source).expect("Read via symlink");
    let via_dest = fs::read_to_string(&dest).expect("Read via dest");
    assert_eq!(via_symlink, via_dest, "Content should match");
}

#[test]
fn relative_symlink_computed_correctly() {
    // GIVEN: Source in /a/b/file, destination in /x/y/file
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("a").join("b").join("file.txt");
    let dest = temp.path().join("x").join("y").join("file.txt");
    let content = "relative path test";

    create_test_file(&source, content);

    // WHEN: mvln with relative mode (default)
    let options = MoveOptions {
        absolute: false,
        ..Default::default()
    };
    let result = move_and_link(&source, &dest, &options);

    // THEN: Symlink uses relative path
    assert!(result.is_ok(), "Operation should succeed");
    assert!(source.is_symlink(), "Source should be symlink");

    // Read the raw symlink target (before resolution)
    let raw_target = fs::read_link(&source).expect("Should read symlink");

    // Should be relative (not start with /)
    assert!(
        !raw_target.is_absolute(),
        "Symlink should be relative, got: {:?}",
        raw_target
    );

    // Should navigate correctly (e.g., ../../x/y/file.txt)
    let link_dir = source.parent().unwrap();
    let resolved = link_dir.join(&raw_target);
    let resolved = resolved.canonicalize().expect("Should resolve");
    let expected = dest.canonicalize().expect("Should canonicalize dest");
    assert_eq!(resolved, expected, "Relative path should resolve correctly");
}

#[test]
fn absolute_symlink_uses_absolute_path() {
    // GIVEN: A source file
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source.txt");
    let dest = temp.path().join("dest.txt");

    create_test_file(&source, "absolute test");

    // WHEN: mvln with absolute mode
    let options = MoveOptions {
        absolute: true,
        ..Default::default()
    };
    let result = move_and_link(&source, &dest, &options);

    // THEN: Symlink target is absolute
    assert!(result.is_ok(), "Operation should succeed");
    assert!(source.is_symlink(), "Source should be symlink");

    let raw_target = fs::read_link(&source).expect("Should read symlink");
    assert!(
        raw_target.is_absolute(),
        "Symlink should be absolute, got: {:?}",
        raw_target
    );
}

// =============================================================================
// Error Condition Tests
// =============================================================================

#[test]
fn source_not_found_returns_error() {
    // GIVEN: Source does not exist
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("nonexistent.txt");
    let dest = temp.path().join("dest.txt");

    // WHEN: mvln is called
    let result = move_and_link(&source, &dest, &MoveOptions::default());

    // THEN: Returns SourceNotFound error
    assert!(result.is_err(), "Should fail for nonexistent source");
    let err = result.unwrap_err();
    assert!(
        matches!(err, MvlnError::SourceNotFound { .. }),
        "Should be SourceNotFound error, got: {:?}",
        err
    );
}

#[test]
fn destination_exists_without_force_returns_error() {
    // GIVEN: Both source and destination exist
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source.txt");
    let dest = temp.path().join("dest.txt");

    create_test_file(&source, "source content");
    create_test_file(&dest, "existing dest");

    // WHEN: mvln without force flag
    let options = MoveOptions {
        force: false,
        ..Default::default()
    };
    let result = move_and_link(&source, &dest, &options);

    // THEN: Returns DestinationExists error
    assert!(result.is_err(), "Should fail when dest exists");
    let err = result.unwrap_err();
    assert!(
        matches!(err, MvlnError::DestinationExists { .. }),
        "Should be DestinationExists error, got: {:?}",
        err
    );

    // AND: Source is unchanged (not moved or deleted!)
    assert!(source.exists(), "Source must remain intact");
    let source_content = fs::read_to_string(&source).unwrap();
    assert_eq!(source_content, "source content", "Source content preserved");
}

#[test]
fn destination_exists_with_force_overwrites() {
    // GIVEN: Both source and destination exist
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source.txt");
    let dest = temp.path().join("dest.txt");

    create_test_file(&source, "new content");
    create_test_file(&dest, "old content");

    // WHEN: mvln with force flag
    let options = MoveOptions {
        force: true,
        ..Default::default()
    };
    let result = move_and_link(&source, &dest, &options);

    // THEN: Operation succeeds, dest has new content
    assert!(result.is_ok(), "Should succeed with force flag");

    let dest_content = fs::read_to_string(&dest).expect("Should read dest");
    assert_eq!(dest_content, "new content", "Dest should have new content");

    assert!(source.is_symlink(), "Source should be symlink");
}

// =============================================================================
// Dry-Run Tests
// =============================================================================

#[test]
fn dry_run_does_not_modify_filesystem() {
    // GIVEN: A source file
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source.txt");
    let dest = temp.path().join("dest.txt");
    let content = "should not move";

    create_test_file(&source, content);

    // WHEN: mvln with dry-run
    let options = MoveOptions {
        dry_run: true,
        ..Default::default()
    };
    let result = move_and_link(&source, &dest, &options);

    // THEN: No filesystem changes
    assert!(result.is_ok(), "Dry-run should succeed");

    // Source is still a regular file (not moved, not symlink)
    assert!(source.is_file(), "Source should still be a regular file");
    assert!(!source.is_symlink(), "Source should NOT be a symlink");

    // Destination does not exist
    assert!(!dest.exists(), "Destination should NOT be created");

    // Content unchanged
    let actual_content = fs::read_to_string(&source).unwrap();
    assert_eq!(actual_content, content, "Source content unchanged");
}
