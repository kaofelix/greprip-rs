// Edge case tests for grg and fnd

use std::process::Command;
use std::fs;
use tempfile::TempDir;

fn grg_binary() -> String {
    "./target/release/grg".to_string()
}

fn fnd_binary() -> String {
    "./target/release/fnd".to_string()
}

// Exit codes must match for script compatibility
#[test]
fn test_grg_no_match_exits_1() {
    // grep exits 1 when no match found
    let result = Command::new(grg_binary())
        .args(&["this_pattern_wont_match_xyz", "tests/fixtures/sample.txt"])
        .output()
        .expect("failed to execute grg");
    assert_eq!(result.status.code(), Some(1));
}

#[test]
fn test_grg_match_exits_0() {
    // grep exits 0 when match found
    let result = Command::new(grg_binary())
        .args(&["hello", "tests/fixtures/sample.txt"])
        .output()
        .expect("failed to execute grg");
    assert_eq!(result.status.code(), Some(0));
}

#[test]
fn test_grg_error_exits_2() {
    // grep exits 2 on error (e.g., file not found)
    let result = Command::new(grg_binary())
        .args(&["pattern", "nonexistent_file_xyz.txt"])
        .output()
        .expect("failed to execute grg");
    // rg exits 2 for errors, matching grep behavior
    assert_eq!(result.status.code(), Some(2));
}

#[test]
fn test_fnd_no_match_exits_0() {
    // find exits 0 even when nothing found
    let result = Command::new(fnd_binary())
        .args(&["tests/fixtures", "-name", "nonexistent_xyz.txt"])
        .output()
        .expect("failed to execute fnd");
    // fd exits 0 when no matches (like find)
    assert_eq!(result.status.code(), Some(0));
}

// Filenames with spaces and special characters
#[test]
fn test_grg_file_with_spaces() {
    // grep should handle files with spaces
    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    let test_file = tmp_dir.path().join("file with spaces.txt");
    fs::write(&test_file, "hello world\n").expect("failed to write file");
    
    let result = Command::new(grg_binary())
        .args(&["hello", test_file.to_str().unwrap()])
        .output()
        .expect("failed to execute grg");
    
    assert_eq!(result.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("hello"));
}

#[test]
fn test_fnd_file_with_spaces() {
    // find should handle files with spaces
    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    let test_file = tmp_dir.path().join("file with spaces.txt");
    fs::write(&test_file, "content\n").expect("failed to write file");
    
    let result = Command::new(fnd_binary())
        .args(&[tmp_dir.path().to_str().unwrap(), "-name", "*.txt"])
        .output()
        .expect("failed to execute fnd");
    
    assert_eq!(result.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("file with spaces.txt"));
}

// Common piping patterns used in scripts
#[test]
fn test_fnd_print0_for_xargs() {
    // find -print0 for xargs -0 compatibility
    let result = Command::new(fnd_binary())
        .args(&["tests/fixtures", "-name", "*.txt", "-print0"])
        .output()
        .expect("failed to execute fnd");
    
    assert_eq!(result.status.code(), Some(0));
    // Output should contain null bytes
    assert!(result.stdout.contains(&0x00));
}

#[test]
fn test_grg_null_separated() {
    // grep -l with null separation for xargs
    let result = Command::new(grg_binary())
        .args(&["-l", "--null", "hello", "tests/fixtures/"])
        .output()
        .expect("failed to execute grg");
    
    // rg supports --null for -l output
    assert_eq!(result.status.code(), Some(0));
}

// Symlink handling
#[test]
fn test_fnd_follows_symlinks_with_l() {
    // find -L follows symlinks
    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    
    // Create a file and symlink
    let real_file = tmp_dir.path().join("real.txt");
    fs::write(&real_file, "content\n").expect("failed to write file");
    
    let link_dir = tmp_dir.path().join("links");
    fs::create_dir(&link_dir).expect("failed to create dir");
    let symlink = link_dir.join("link.txt");
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink as make_symlink;
        make_symlink(&real_file, &symlink).expect("failed to create symlink");
    }
    
    // Without -L, fd shows symlinks as type l
    let result = Command::new(fnd_binary())
        .args(&[link_dir.to_str().unwrap(), "-type", "l"])
        .output()
        .expect("failed to execute fnd");
    
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("link.txt"));
}

// Empty inputs and edge cases
#[test]
fn test_grg_empty_pattern() {
    // Empty pattern should match all lines
    let result = Command::new(grg_binary())
        .args(&["", "tests/fixtures/sample.txt"])
        .output()
        .expect("failed to execute grg");
    
    // rg with empty pattern matches all lines
    assert_eq!(result.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.lines().count() > 1);
}

#[test]
fn test_fnd_current_dir_implicit() {
    // find with just . should list all
    let result = Command::new(fnd_binary())
        .args(&["tests/fixtures"])
        .output()
        .expect("failed to execute fnd");
    
    assert_eq!(result.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("sample.txt"));
}
