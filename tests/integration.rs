use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper to get the mvln binary command
fn mvln_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mvln"))
}

#[test]
fn test_single_file_move_and_link() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("file.txt");
    let dest_dir = tmp.path().join("dest");

    fs::write(&src, "test content").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    mvln_cmd().arg(&src).arg(&dest_dir).assert().success();

    // Original path should be a symlink
    assert!(src.is_symlink());
    assert!(src.exists()); // Symlink resolves successfully

    // Destination should contain the real file
    let dest_file = dest_dir.join("file.txt");
    assert!(dest_file.exists());
    assert!(!dest_file.is_symlink());
    assert_eq!(fs::read_to_string(&dest_file).unwrap(), "test content");
}

#[test]
fn test_glob_pattern_multiple_files() {
    let tmp = TempDir::new().unwrap();
    let file1 = tmp.path().join("a.txt");
    let file2 = tmp.path().join("b.txt");
    let file3 = tmp.path().join("c.log");
    let dest_dir = tmp.path().join("dest");

    fs::write(&file1, "a").unwrap();
    fs::write(&file2, "b").unwrap();
    fs::write(&file3, "c").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    mvln_cmd()
        .current_dir(tmp.path())
        .arg("*.txt")
        .arg(&dest_dir)
        .assert()
        .success();

    // Both .txt files should be symlinks
    assert!(file1.is_symlink());
    assert!(file2.is_symlink());
    // .log file should remain untouched
    assert!(!file3.is_symlink());

    // Destination should contain real files
    assert!(dest_dir.join("a.txt").exists());
    assert!(dest_dir.join("b.txt").exists());
    assert!(!dest_dir.join("c.log").exists());
}

#[test]
fn test_directory_move_with_whole_dir() {
    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src_dir");
    let dest_dir = tmp.path().join("dest");
    let src_file = src_dir.join("file.txt");

    fs::create_dir(&src_dir).unwrap();
    fs::write(&src_file, "content").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    mvln_cmd()
        .arg("-w")
        .arg(&src_dir)
        .arg(&dest_dir)
        .assert()
        .success();

    // Source dir should be a symlink
    assert!(src_dir.is_symlink());

    // Destination should contain the moved directory
    let moved_dir = dest_dir.join("src_dir");
    assert!(moved_dir.exists());
    assert!(moved_dir.is_dir());
    assert!(moved_dir.join("file.txt").exists());
}

#[test]
fn test_relative_symlink_flag() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("file.txt");
    let dest_dir = tmp.path().join("dest");

    fs::write(&src, "test").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    mvln_cmd()
        .arg("-r")
        .arg(&src)
        .arg(&dest_dir)
        .assert()
        .success();

    // Check symlink is relative
    let link_target = fs::read_link(&src).unwrap();
    assert!(link_target.is_relative());
}

#[test]
fn test_absolute_symlink_flag() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("file.txt");
    let dest_dir = tmp.path().join("dest");

    fs::write(&src, "test").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    mvln_cmd()
        .arg("-a")
        .arg(&src)
        .arg(&dest_dir)
        .assert()
        .success();

    // Check symlink is absolute
    let link_target = fs::read_link(&src).unwrap();
    assert!(link_target.is_absolute());
}

#[test]
fn test_verbose_output() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("file.txt");
    let dest_dir = tmp.path().join("dest");

    fs::write(&src, "test").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    mvln_cmd()
        .arg("-v")
        .arg(&src)
        .arg(&dest_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("file.txt"));
}

#[test]
fn test_missing_source_fails() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("nonexistent.txt");
    let dest_dir = tmp.path().join("dest");

    fs::create_dir(&dest_dir).unwrap();

    mvln_cmd().arg(&src).arg(&dest_dir).assert().failure();
}

#[test]
fn test_destination_created_if_not_exists() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("file.txt");
    let dest_path = tmp.path().join("nonexistent_dest");

    fs::write(&src, "test").unwrap();

    mvln_cmd().arg(&src).arg(&dest_path).assert().success();

    // Should move the file to the destination path
    assert!(src.is_symlink());
    assert!(dest_path.exists());
}

#[test]
fn test_no_args_shows_help() {
    mvln_cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_help_flag() {
    mvln_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Move files and create symlinks at original locations",
        ));
}

#[test]
fn test_version_flag() {
    mvln_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("mvls"));
}

#[test]
fn test_multiple_sources_to_directory() {
    let tmp = TempDir::new().unwrap();
    let file1 = tmp.path().join("file1.txt");
    let file2 = tmp.path().join("file2.txt");
    let dest_dir = tmp.path().join("dest");

    fs::write(&file1, "content1").unwrap();
    fs::write(&file2, "content2").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    mvln_cmd()
        .arg(&file1)
        .arg(&file2)
        .arg(&dest_dir)
        .assert()
        .success();

    // Both files should be symlinks
    assert!(file1.is_symlink());
    assert!(file2.is_symlink());

    // Destination should contain both files
    assert!(dest_dir.join("file1.txt").exists());
    assert!(dest_dir.join("file2.txt").exists());
}

#[test]
fn test_symlink_resolution() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("file.txt");
    let dest_dir = tmp.path().join("dest");

    fs::write(&src, "original content").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    mvln_cmd().arg(&src).arg(&dest_dir).assert().success();

    // Verify we can read through the symlink
    assert_eq!(fs::read_to_string(&src).unwrap(), "original content");

    // Verify the symlink points to the right place
    let link_target = fs::read_link(&src).unwrap();
    let resolved = if link_target.is_absolute() {
        link_target
    } else {
        tmp.path().join(link_target)
    };

    assert!(resolved.exists());
}
