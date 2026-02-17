// Unit tests for grep → rg argument translation

use greprip::grg::translate_grep_args;

fn translate(args: &[&str]) -> Vec<String> {
    translate_grep_args(args)
        .into_iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect()
}

// Basic patterns
#[test]
fn test_simple_pattern_and_file() {
    assert_eq!(
        translate(&["hello", "file.txt"]),
        &["hello", "file.txt"]
    );
}

#[test]
fn test_pattern_only() {
    assert_eq!(translate(&["hello"]), &["hello"]);
}

// Common flags
#[test]
fn test_case_insensitive() {
    assert_eq!(
        translate(&["-i", "hello", "file.txt"]),
        &["-i", "hello", "file.txt"]
    );
}

#[test]
fn test_line_numbers() {
    assert_eq!(
        translate(&["-n", "hello", "file.txt"]),
        &["-n", "hello", "file.txt"]
    );
}

#[test]
fn test_recursive() {
    // rg is recursive by default, so -r should be dropped
    assert_eq!(
        translate(&["-r", "hello", "dir/"]),
        &["hello", "dir/"]
    );
}

#[test]
fn test_invert_match() {
    assert_eq!(
        translate(&["-v", "hello", "file.txt"]),
        &["-v", "hello", "file.txt"]
    );
}

#[test]
fn test_word_boundary() {
    assert_eq!(
        translate(&["-w", "foo", "file.txt"]),
        &["-w", "foo", "file.txt"]
    );
}

#[test]
fn test_files_with_matches() {
    assert_eq!(
        translate(&["-l", "hello", "dir/"]),
        &["-l", "hello", "dir/"]
    );
}

#[test]
fn test_count() {
    assert_eq!(
        translate(&["-c", "hello", "file.txt"]),
        &["-c", "hello", "file.txt"]
    );
}

// Combined flags
#[test]
fn test_recursive_case_insensitive() {
    // -ri should drop -r (rg default) but keep -i
    let result = translate(&["-ri", "hello", "dir/"]);
    assert!(result.contains(&"-i".to_string()));
    assert!(result.contains(&"hello".to_string()));
    assert!(result.contains(&"dir/".to_string()));
}

#[test]
fn test_multiple_separate_flags() {
    let result = translate(&["-i", "-n", "hello", "file.txt"]);
    assert!(result.contains(&"-i".to_string()));
    assert!(result.contains(&"-n".to_string()));
    assert!(result.contains(&"hello".to_string()));
}

// Extended regex
#[test]
fn test_extended_regex_flag() {
    // grep -E is default in rg, so we can drop it
    assert_eq!(
        translate(&["-E", "foo|bar", "file.txt"]),
        &["foo|bar", "file.txt"]
    );
}

// Include/exclude patterns
#[test]
fn test_include_pattern() {
    let result = translate(&["--include=*.py", "-r", "hello", "dir/"]);
    assert!(result.contains(&"-g".to_string()) || result.contains(&"--glob".to_string()));
    assert!(result.join(" ").contains("*.py"));
}

#[test]
fn test_exclude_pattern() {
    let result = translate(&["--exclude=*.pyc", "-r", "hello", "dir/"]);
    assert!(result.contains(&"-g".to_string()) || result.contains(&"--glob".to_string()));
    let joined = result.join(" ");
    assert!(joined.contains("!*.pyc") || joined.contains("*.pyc"));
}

// Context lines
#[test]
fn test_after_context() {
    let result = translate(&["-A", "3", "hello", "file.txt"]);
    assert!(result.contains(&"-A".to_string()));
    assert!(result.contains(&"3".to_string()));
}

#[test]
fn test_before_context() {
    let result = translate(&["-B", "2", "hello", "file.txt"]);
    assert!(result.contains(&"-B".to_string()));
    assert!(result.contains(&"2".to_string()));
}

#[test]
fn test_context_both() {
    let result = translate(&["-C", "5", "hello", "file.txt"]);
    assert!(result.contains(&"-C".to_string()));
    assert!(result.contains(&"5".to_string()));
}

#[test]
fn test_combined_context_short_form() {
    // grep allows -3 as shorthand for -C 3
    let result = translate(&["-3", "hello", "file.txt"]);
    assert!(result.contains(&"-C".to_string()));
    assert!(result.contains(&"3".to_string()));
}

// Explicit pattern
#[test]
fn test_single_pattern_with_e() {
    let result = translate(&["-e", "hello", "file.txt"]);
    assert!(result.contains(&"-e".to_string()));
    assert!(result.contains(&"hello".to_string()));
}

#[test]
fn test_multiple_patterns_with_e() {
    let result = translate(&["-e", "hello", "-e", "world", "file.txt"]);
    assert_eq!(result.iter().filter(|&x| x == "-e").count(), 2);
    assert!(result.contains(&"hello".to_string()));
    assert!(result.contains(&"world".to_string()));
}

// Pattern file
#[test]
fn test_patterns_from_file() {
    let result = translate(&["-f", "patterns.txt", "file.txt"]);
    assert!(result.contains(&"-f".to_string()));
    assert!(result.contains(&"patterns.txt".to_string()));
}

// Quiet and suppress
#[test]
fn test_quiet_mode() {
    let result = translate(&["-q", "hello", "file.txt"]);
    assert!(result.contains(&"-q".to_string()));
}

#[test]
fn test_suppress_errors() {
    // grep -s suppresses errors about nonexistent files
    // rg uses --no-messages
    let result = translate(&["-s", "hello", "file.txt"]);
    assert!(result.contains(&"--no-messages".to_string()));
}

// Color handling
#[test]
fn test_color_always() {
    let result = translate(&["--color=always", "hello", "file.txt"]);
    assert!(result.contains(&"--color=always".to_string()));
}

#[test]
fn test_color_never() {
    let result = translate(&["--color=never", "hello", "file.txt"]);
    assert!(result.contains(&"--color=never".to_string()));
}

#[test]
fn test_color_auto() {
    let result = translate(&["--color=auto", "hello", "file.txt"]);
    assert!(result.contains(&"--color=auto".to_string()));
}

#[test]
fn test_color_flag_without_value() {
    // --color alone means --color=always in grep
    let result = translate(&["--color", "hello", "file.txt"]);
    assert!(result.contains(&"--color=always".to_string()) || result.contains(&"--color".to_string()));
}

// Fixed strings
#[test]
fn test_fixed_strings() {
    // grep -F treats pattern as literal, rg uses -F too
    let result = translate(&["-F", "hello.world", "file.txt"]);
    assert!(result.contains(&"-F".to_string()));
}

// Perl regex
#[test]
fn test_perl_regex() {
    // grep -P for PCRE, rg uses -P too
    let result = translate(&["-P", r"\bhello\b", "file.txt"]);
    assert!(result.contains(&"-P".to_string()));
}

// Long options
#[test]
fn test_ignore_case_long() {
    let result = translate(&["--ignore-case", "hello", "file.txt"]);
    assert!(result.contains(&"-i".to_string()) || result.contains(&"--ignore-case".to_string()));
}

#[test]
fn test_line_number_long() {
    let result = translate(&["--line-number", "hello", "file.txt"]);
    assert!(result.contains(&"-n".to_string()) || result.contains(&"--line-number".to_string()));
}

#[test]
fn test_recursive_long() {
    let result = translate(&["--recursive", "hello", "dir/"]);
    assert!(result.contains(&"hello".to_string()));  // -r should be dropped
}

#[test]
fn test_word_regexp_long() {
    let result = translate(&["--word-regexp", "hello", "file.txt"]);
    assert!(result.contains(&"-w".to_string()) || result.contains(&"--word-regexp".to_string()));
}

#[test]
fn test_files_with_matches_long() {
    let result = translate(&["--files-with-matches", "hello", "dir/"]);
    assert!(result.contains(&"-l".to_string()) || result.contains(&"--files-with-matches".to_string()));
}

#[test]
fn test_invert_match_long() {
    let result = translate(&["--invert-match", "hello", "file.txt"]);
    assert!(result.contains(&"-v".to_string()) || result.contains(&"--invert-match".to_string()));
}

#[test]
fn test_count_long() {
    let result = translate(&["--count", "hello", "file.txt"]);
    assert!(result.contains(&"-c".to_string()) || result.contains(&"--count".to_string()));
}
