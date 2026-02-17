# greprip (Rust)

High-performance drop-in replacements for `grep` and `find` using Rust's `ripgrep` and `fd`.

## Overview

This is a Rust port of the [greprip](https://github.com/kaofelix/greprip) project, providing two binaries:

- **grg**: Translates `grep` commands to `rg` (ripgrep)
- **fnd**: Translates `find` commands to `fd` (fd-find)

## Why?

LLM coding agents default to using `grep` and `find` because they're universal POSIX tools. But modern Rust alternatives like `ripgrep` and `fd` are significantly faster. **greprip** acts as a transparent layer that:

1. Accepts standard `grep`/`find` syntax
2. Translates arguments to `rg`/`fd` equivalents
3. Executes the faster tool transparently

## Installation

### Prerequisites

```bash
# Install rg and fd (macOS)
brew install ripgrep fd

# Install rg and fd (Linux)
# Ubuntu/Debian: apt install ripgrep fd-find
# Arch: pacman -S ripgrep fd
```

### Build from source

```bash
cargo build --release
```

The binaries will be available in `./target/release/grg` and `./target/release/fnd`.

## Usage

### As drop-in replacements

Add to your shell configuration:

```bash
# In ~/.bashrc or ~/.zshrc
alias grep='/path/to/grg'
alias find='/path/to/fnd'
```

### With Pi (coding agent)

Add to `~/.pi/agent/settings.json`:

```json
{
  "shellCommandPrefix": "grep() { grg \"$@\"; }; find() { fnd \"$@\"; };"
}
```

## Supported Features

### grg (grep → rg)

#### Basic Flags
- `-i`, `--ignore-case`: Case insensitive search
- `-n`, `--line-number`: Show line numbers
- `-v`, `--invert-match`: Invert matches
- `-w`, `--word-regexp`: Word boundary matching
- `-l`, `--files-with-matches`: Show only filenames
- `-c`, `--count`: Count matches
- `-o`, `--only-matching`: Show only matching parts
- `-h`, `-H`: Filename control
- `-q`, `--quiet`: Quiet mode

#### Recursive Search
- `-r`, `-R`, `--recursive`: Recursive search (dropped - rg default)

#### Regex Support
- `-E`, `--extended-regexp`: Extended regex (dropped - rg default)
- `-F`, `--fixed-strings`: Literal string matching
- `-P`, `--perl-regexp`: PCRE support

#### Context Lines
- `-A NUM`: After context
- `-B NUM`: Before context
- `-C NUM`: Context both sides
- `-NUM`: Shorthand for -C NUM

#### Pattern Control
- `-e PATTERN`: Explicit pattern
- `-f FILE`: Patterns from file
- Multiple `-e` patterns supported

#### File Filtering
- `--include=PATTERN`: Include files matching pattern
- `--exclude=PATTERN`: Exclude files matching pattern
- `--exclude-dir=PATTERN`: Exclude directories

#### Output Control
- `--color[=WHEN]`: Color control
- `-s`: Suppress errors (maps to `--no-messages`)
- `--null`: Null-separated output

#### Long Options
All common long options are supported (e.g., `--ignore-case`, `--word-regexp`, etc.)

### fnd (find → fd)

#### Basic Usage
- Path arguments (e.g., `fnd /path`)
- Current directory (e.g., `fnd .`)

#### Name Matching
- `-name PATTERN`: Match by name (glob pattern)
- `-iname PATTERN`: Case-insensitive name matching
- `! -name PATTERN`: Exclude pattern

#### Type Filtering
- `-type f`: Files only
- `-type d`: Directories only
- `-type l`: Symlinks only

#### Depth Control
- `-maxdepth N`: Maximum depth
- `-mindepth N`: Minimum depth

#### Execution
- `-exec cmd {} \;`: Execute command per file
- `-exec cmd {} +`: Batch execution
- `-print`: Print results (implicit)
- `-print0`: Null-separated output

#### Other Options
- `-L`: Follow symlinks
- `-path PATTERN -prune`: Exclude paths
- Hidden files included by default (fd `-H` added)

## Testing

```bash
# Run all unit tests
cargo test

# Run specific test suites
cargo test --test test_grep_to_rg
cargo test --test test_find_to_fd
cargo test --test test_edge_cases

# Run acceptance tests
GRG="./target/release/grg" ./tests/acceptance/test_grg.sh
FND="./target/release/fnd" ./tests/acceptance/test_fnd.sh
```

### Test Coverage

- **68 unit tests** for grep/rg translation
- **26 unit tests** for find/fd translation
- **11 edge case tests** for error handling and special scenarios
- **32 acceptance tests** comparing actual grep/find vs grg/fnd output

Total: **137 tests**

## Performance

The Rust implementation provides:
- Zero-cost argument translation
- No runtime dependencies (statically linked)
- Minimal memory footprint
- Fast startup time

## Differences from Original

This Rust port is a complete rewrite with:
- ✅ All unit tests ported from Python
- ✅ All acceptance tests passing
- ✅ Same behavior as Python version
- ✅ Better performance (native binary, no interpreter)
- ✅ Smaller deployment footprint

## Development

### Project Structure

```
greprip-rs/
├── src/
│   ├── lib.rs              # Library exports
│   ├── grg.rs              # grep → rg translator
│   ├── fnd.rs              # find → fd translator
│   └── bin/
│       ├── grg.rs          # grg binary
│       └── fnd.rs          # fnd binary
├── tests/
│   ├── fixtures/           # Test files
│   ├── acceptance/         # Acceptance tests (bash)
│   ├── test_grep_to_rg.rs  # grg unit tests
│   ├── test_find_to_fd.rs  # fnd unit tests
│   └── test_edge_cases.rs  # Edge case tests
└── Cargo.toml
```

### Adding New Translations

1. Add unit tests to `tests/test_grep_to_rg.rs` or `tests/test_find_to_fd.rs`
2. Implement translation in `src/grg.rs` or `src/fnd.rs`
3. Update documentation

## Known Limitations

- Complex find expressions (e.g., `-a`, `-o` operators) are simplified
- Some obscure grep flags may not be supported
- fd output doesn't include search root (find does)

## License

MIT

## Acknowledgments

This is a Rust port of the original [greprip](https://github.com/kaofelix/greprip) project by [@kaofelix](https://github.com/kaofelix).
