use std::ffi::OsString;

/// Translate find command-line arguments to fd equivalents.
pub fn translate_find_args(args: &[&str]) -> Vec<OsString> {
    let mut result = Vec::new();
    let mut paths = Vec::new();
    let mut pattern = None;
    let mut exec_args = Vec::new();
    let mut i = 0;

    // fd needs -H to show hidden files (find shows them by default)
    result.push(OsString::from("-H"));

    while i < args.len() {
        let arg = args[i];

        // Path arguments (don't start with -)
        if !arg.starts_with('-') && arg != "!" && arg != "(" && arg != ")" {
            // Check if it looks like a path (first positional args before options)
            if i == 0 || !args[..i].iter().any(|a| a.starts_with('-')) {
                paths.push(arg);
                i += 1;
                continue;
            }
        }

        // -name PATTERN → -g PATTERN (glob)
        if arg == "-name" && i + 1 < args.len() {
            pattern = Some(args[i + 1]);
            result.push(OsString::from("-g"));
            result.push(OsString::from(args[i + 1]));
            i += 2;
            continue;
        }

        // -iname PATTERN → -i -g PATTERN (case insensitive glob)
        if arg == "-iname" && i + 1 < args.len() {
            pattern = Some(args[i + 1]);
            result.push(OsString::from("-i"));
            result.push(OsString::from("-g"));
            result.push(OsString::from(args[i + 1]));
            i += 2;
            continue;
        }

        // -type TYPE
        if arg == "-type" && i + 1 < args.len() {
            result.push(OsString::from("-t"));
            result.push(OsString::from(args[i + 1]));
            i += 2;
            continue;
        }

        // -maxdepth N → -d N
        if arg == "-maxdepth" && i + 1 < args.len() {
            result.push(OsString::from("-d"));
            result.push(OsString::from(args[i + 1]));
            i += 2;
            continue;
        }

        // -mindepth N → --min-depth N
        if arg == "-mindepth" && i + 1 < args.len() {
            result.push(OsString::from("--min-depth"));
            result.push(OsString::from(args[i + 1]));
            i += 2;
            continue;
        }

        // -print0 → -0
        if arg == "-print0" {
            result.push(OsString::from("-0"));
            i += 1;
            continue;
        }

        // -print is default, skip it
        if arg == "-print" {
            i += 1;
            continue;
        }

        // ! -name PATTERN → -E PATTERN (exclude)
        if arg == "!" && i + 2 < args.len() && args[i + 1] == "-name" {
            result.push(OsString::from("-E"));
            result.push(OsString::from(args[i + 2]));
            i += 3;
            continue;
        }

        // -path PATTERN -prune -o ... -print → -E PATTERN (simplified)
        if arg == "-path" && i + 1 < args.len() {
            let path_pattern = args[i + 1];
            // Check if followed by -prune
            if i + 2 < args.len() && args[i + 2] == "-prune" {
                // Extract the directory name from pattern like "*/.git"
                let exclude_pattern = path_pattern.replace("*/", "").replace("*", "");
                result.push(OsString::from("-E"));
                result.push(OsString::from(exclude_pattern));
                i += 3;
                // Skip -o if present
                if i < args.len() && args[i] == "-o" {
                    i += 1;
                }
                continue;
            }
            i += 1;
            continue;
        }

        // -prune alone (already handled above, skip)
        if arg == "-prune" {
            i += 1;
            continue;
        }

        // -o (or) - skip, we handle the main expression
        if arg == "-o" {
            i += 1;
            continue;
        }

        // -exec cmd {} \; → -x cmd
        // -exec cmd {} + → -X cmd (batch mode)
        if arg == "-exec" {
            let mut cmd_args = Vec::new();
            i += 1;
            let mut batch_mode = false;
            while i < args.len() {
                if args[i] == ";" {
                    break;
                } else if args[i] == "+" {
                    batch_mode = true;
                    break;
                } else if args[i] == "{}" {
                    // fd uses {} too, but handles it automatically
                    cmd_args.push(OsString::from("{}"));
                } else {
                    cmd_args.push(OsString::from(args[i]));
                }
                i += 1;
            }
            i += 1; // Skip the terminator

            // Store exec args to add at the very end (after paths)
            if batch_mode {
                exec_args = vec![OsString::from("-X")];
                exec_args.extend(cmd_args);
            } else {
                exec_args = vec![OsString::from("-x")];
                exec_args.extend(cmd_args);
            }
            continue;
        }

        // -L (follow symlinks) → -L
        if arg == "-L" {
            result.push(OsString::from("-L"));
            i += 1;
            continue;
        }

        // Unknown args - skip for now
        i += 1;
    }

    // fd requires a pattern - if none specified via -name/-iname, use "." (match all)
    if pattern.is_none() {
        result.push(OsString::from("."));
    }

    // Add paths (fd takes paths after pattern/options, but before -x/-X)
    if !paths.is_empty() {
        for path in paths {
            result.push(OsString::from(path));
        }
    }

    // Add exec args last (everything after -x/-X is treated as the command)
    if !exec_args.is_empty() {
        result.extend(exec_args);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translate(args: &[&str]) -> Vec<String> {
        translate_find_args(args)
            .into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect()
    }

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

    #[test]
    fn test_not_name() {
        let result = translate(&[".", "!", "-name", "*.pyc"]);
        assert!(result.contains(&"-E".to_string()) || result.contains(&"--exclude".to_string()));
        let joined = result.join(" ");
        assert!(joined.contains("*.pyc"));
    }

    #[test]
    fn test_print0() {
        let result = translate(&[".", "-print0"]);
        assert!(result.contains(&"-0".to_string()) || result.contains(&"--print0".to_string()));
    }

    #[test]
    fn test_name_and_type() {
        let result = translate(&[".", "-name", "*.py", "-type", "f"]);
        assert!(result.contains(&"-t".to_string()) || result.contains(&"--type".to_string()));
        let joined = result.join(" ");
        assert!(joined.contains("*.py") || joined.contains(".py"));
    }

    #[test]
    fn test_include_hidden_default() {
        // find includes hidden by default, fd doesn't
        // We need to add -H to fd to match find behavior
        let result = translate(&[".", "-name", "*.txt"]);
        assert!(result.contains(&"-H".to_string()) || result.contains(&"--hidden".to_string()));
    }

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
    fn test_follow_symlinks() {
        let result = translate(&["-L", ".", "-name", "*.txt"]);
        assert!(result.contains(&"-L".to_string()));
    }
}
