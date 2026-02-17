// Unit tests for find → fd argument translation

use greprip::fnd::translate_find_args;

fn translate(args: &[&str]) -> Vec<String> {
    translate_find_args(args)
        .into_iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect()
}

// Basic usage
#[test]
fn test_path_only() {
    let result = translate(&["/some/path"]);
    assert!(result.contains(&"/some/path".to_string()));
}

#[test]
fn test_current_dir() {
    let result = translate(&["."]);
    assert!(result.contains(&".".to_string()));
}

// Name patterns
#[test]
fn test_name_pattern() {
    let result = translate(&[".", "-name", "*.txt"]);
    let joined = result.join(" ");
    assert!(joined.contains("*.txt") || joined.contains(".txt"));
}

#[test]
fn test_iname_pattern() {
    let result = translate(&[".", "-iname", "*.TXT"]);
    assert!(result.contains(&"-i".to_string()));
}

#[test]
fn test_name_with_path() {
    let result = translate(&["/path", "-name", "*.py"]);
    assert!(result.contains(&"/path".to_string()));
    let joined = result.join(" ");
    assert!(joined.contains("*.py") || joined.contains(".py"));
}

// Type filter
#[test]
fn test_type_file() {
    let result = translate(&[".", "-type", "f"]);
    assert!(result.contains(&"-t".to_string()) || result.contains(&"--type".to_string()));
    assert!(result.contains(&"f".to_string()));
}

#[test]
fn test_type_directory() {
    let result = translate(&[".", "-type", "d"]);
    assert!(result.contains(&"-t".to_string()) || result.contains(&"--type".to_string()));
    assert!(result.contains(&"d".to_string()));
}

#[test]
fn test_type_symlink() {
    let result = translate(&[".", "-type", "l"]);
    assert!(result.contains(&"-t".to_string()) || result.contains(&"--type".to_string()));
    assert!(result.contains(&"l".to_string()));
}

// Depth control
#[test]
fn test_maxdepth() {
    let result = translate(&[".", "-maxdepth", "2"]);
    assert!(result.contains(&"-d".to_string()) || result.contains(&"--max-depth".to_string()));
    assert!(result.contains(&"2".to_string()));
}

#[test]
fn test_mindepth() {
    let result = translate(&[".", "-mindepth", "1"]);
    assert!(result.contains(&"--min-depth".to_string()));
    assert!(result.contains(&"1".to_string()));
}

// Exclude patterns
#[test]
fn test_not_name() {
    let result = translate(&[".", "!", "-name", "*.pyc"]);
    assert!(result.contains(&"-E".to_string()) || result.contains(&"--exclude".to_string()));
    let joined = result.join(" ");
    assert!(joined.contains("*.pyc"));
}

#[test]
fn test_prune_directory() {
    let result = translate(&[".", "-path", "*/.git", "-prune", "-o", "-type", "f", "-print"]);
    // Should exclude .git somehow
    let joined = result.join(" ");
    assert!(joined.contains("-E") || joined.contains("--exclude") || joined.contains(".git"));
}

// Print actions
#[test]
fn test_print() {
    // -print is implicit in fd, should not cause issues
    let result = translate(&[".", "-name", "*.txt", "-print"]);
    let joined = result.join(" ");
    assert!(joined.contains("*.txt") || joined.contains(".txt"));
}

#[test]
fn test_print0() {
    let result = translate(&[".", "-print0"]);
    assert!(result.contains(&"-0".to_string()) || result.contains(&"--print0".to_string()));
}

// Combinations
#[test]
fn test_name_and_type() {
    let result = translate(&[".", "-name", "*.py", "-type", "f"]);
    assert!(result.contains(&"-t".to_string()) || result.contains(&"--type".to_string()));
    let joined = result.join(" ");
    assert!(joined.contains("*.py") || joined.contains(".py"));
}

#[test]
fn test_path_name_maxdepth() {
    let result = translate(&["/path", "-maxdepth", "3", "-name", "*.js"]);
    assert!(result.contains(&"/path".to_string()));
    assert!(result.contains(&"-d".to_string()) || result.contains(&"--max-depth".to_string()));
    assert!(result.contains(&"3".to_string()));
}

// Hidden files
#[test]
fn test_include_hidden_default() {
    // find includes hidden by default, fd doesn't
    // We need to add -H to fd to match find behavior
    let result = translate(&[".", "-name", "*.txt"]);
    assert!(result.contains(&"-H".to_string()) || result.contains(&"--hidden".to_string()));
}

// Exec
#[test]
fn test_exec_single() {
    let result = translate(&[".", "-name", "*.py", "-exec", "wc", "-l", "{}", ";"]);
    assert!(result.contains(&"-x".to_string()));
    assert!(result.contains(&"wc".to_string()));
    assert!(result.contains(&"-l".to_string()));
}

#[test]
fn test_exec_batch() {
    let result = translate(&[".", "-type", "f", "-exec", "chmod", "644", "{}", "+"]);
    assert!(result.contains(&"-X".to_string()));
    assert!(result.contains(&"chmod".to_string()));
    assert!(result.contains(&"644".to_string()));
}

#[test]
fn test_exec_with_grep() {
    let result = translate(&[".", "-name", "*.py", "-exec", "grep", "pattern", "{}", ";"]);
    assert!(result.contains(&"-x".to_string()));
    assert!(result.contains(&"grep".to_string()));
    assert!(result.contains(&"pattern".to_string()));
}

// Follow symlinks
#[test]
fn test_follow_symlinks() {
    let result = translate(&["-L", ".", "-name", "*.txt"]);
    assert!(result.contains(&"-L".to_string()));
}
