use std::ffi::OsString;

/// Translate grep command-line arguments to rg equivalents.
pub fn translate_grep_args(args: &[&str]) -> Vec<OsString> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = args[i];

        // Handle numeric context shorthand: -3 means -C 3
        if arg.starts_with('-') && arg.len() > 1 && arg[1..].chars().all(|c| c.is_ascii_digit()) {
            let num = &arg[1..];
            result.push(OsString::from("-C"));
            result.push(OsString::from(num));
            i += 1;
            continue;
        }

        // Handle flags to drop (rg defaults or not needed)
        if arg == "-r" || arg == "-R" || arg == "-E" || arg == "-G" {
            i += 1;
            continue;
        }

        // Handle -s (suppress errors) -> --no-messages
        if arg == "-s" {
            result.push(OsString::from("--no-messages"));
            i += 1;
            continue;
        }

        // Handle --include=PATTERN
        if arg.starts_with("--include=") {
            let pattern = arg.splitn(2, '=').nth(1).unwrap_or("");
            result.push(OsString::from("-g"));
            result.push(OsString::from(pattern));
            i += 1;
            continue;
        }

        // Handle --exclude=PATTERN
        if arg.starts_with("--exclude=") {
            let pattern = arg.splitn(2, '=').nth(1).unwrap_or("");
            result.push(OsString::from("-g"));
            result.push(OsString::from(format!("!{}", pattern)));
            i += 1;
            continue;
        }

        // Handle --exclude-dir=PATTERN
        if arg.starts_with("--exclude-dir=") {
            let pattern = arg.splitn(2, '=').nth(1).unwrap_or("");
            result.push(OsString::from("-g"));
            result.push(OsString::from(format!("!{}/", pattern)));
            i += 1;
            continue;
        }

        // Handle --color variants
        if arg == "--color" {
            result.push(OsString::from("--color=always"));
            i += 1;
            continue;
        }
        if arg.starts_with("--color=") {
            result.push(OsString::from(arg));
            i += 1;
            continue;
        }

        // Handle combined short flags like -ri, -rni
        if arg.starts_with('-') && !arg.starts_with("--") && arg.len() > 2 && !arg[1..].starts_with(|c: char| c.is_ascii_digit()) {
            // Expand combined flags
            for (_idx, c) in arg[1..].char_indices() {
                let flag = format!("-{}", c);
                
                // Drop certain flags
                if flag == "-r" || flag == "-R" || flag == "-E" || flag == "-G" {
                    continue;
                }
                
                // Handle -s
                if flag == "-s" {
                    result.push(OsString::from("--no-messages"));
                    continue;
                }
                
                // Keep other flags
                result.push(OsString::from(flag));
            }
            i += 1;
            continue;
        }

        // Handle identical flags (no argument needed)
        if ["-i", "-n", "-v", "-w", "-l", "-c", "-o", "-h", "-H", "-q", "-F", "-P", "--null"].contains(&arg) {
            result.push(arg.into());
            i += 1;
            continue;
        }

        // Handle flags with arguments
        if ["-A", "-B", "-C", "-e", "-f", "-m"].contains(&arg) {
            result.push(arg.into());
            if i + 1 < args.len() {
                i += 1;
                result.push(args[i].into());
            }
            i += 1;
            continue;
        }

        // Handle long options
        if arg.starts_with("--") {
            // Map long options to short equivalents
            let mapped = match arg {
                "--ignore-case" => Some("-i"),
                "--line-number" => Some("-n"),
                "--invert-match" => Some("-v"),
                "--word-regexp" => Some("-w"),
                "--files-with-matches" => Some("-l"),
                "--count" => Some("-c"),
                "--only-matching" => Some("-o"),
                "--no-filename" => Some("-h"),
                "--with-filename" => Some("-H"),
                "--quiet" | "--silent" => Some("-q"),
                "--fixed-strings" => Some("-F"),
                "--perl-regexp" => Some("-P"),
                "--extended-regexp" | "--basic-regexp" | "--recursive" => None, // Drop these
                _ => {
                    // Unknown long option, pass through
                    result.push(arg.into());
                    i += 1;
                    continue;
                }
            };
            
            if let Some(short) = mapped {
                result.push(short.into());
            }
            // If mapped is None, we drop it
            i += 1;
            continue;
        }

        // Everything else passes through (pattern, paths, etc.)
        result.push(arg.into());
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translate(args: &[&str]) -> Vec<String> {
        translate_grep_args(args)
            .into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn test_simple_pattern_and_file() {
        assert_eq!(translate(&["hello", "file.txt"]), &["hello", "file.txt"]);
    }

    #[test]
    fn test_pattern_only() {
        assert_eq!(translate(&["hello"]), &["hello"]);
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(translate(&["-i", "hello", "file.txt"]), &["-i", "hello", "file.txt"]);
    }

    #[test]
    fn test_line_numbers() {
        assert_eq!(translate(&["-n", "hello", "file.txt"]), &["-n", "hello", "file.txt"]);
    }

    #[test]
    fn test_recursive() {
        // rg is recursive by default, so -r should be dropped
        assert_eq!(translate(&["-r", "hello", "dir/"]), &["hello", "dir/"]);
    }

    #[test]
    fn test_invert_match() {
        assert_eq!(translate(&["-v", "hello", "file.txt"]), &["-v", "hello", "file.txt"]);
    }

    #[test]
    fn test_word_boundary() {
        assert_eq!(translate(&["-w", "foo", "file.txt"]), &["-w", "foo", "file.txt"]);
    }

    #[test]
    fn test_files_with_matches() {
        assert_eq!(translate(&["-l", "hello", "dir/"]), &["-l", "hello", "dir/"]);
    }

    #[test]
    fn test_count() {
        assert_eq!(translate(&["-c", "hello", "file.txt"]), &["-c", "hello", "file.txt"]);
    }
}
