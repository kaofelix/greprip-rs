#!/usr/bin/env bash
# Acceptance tests for grg (grep → rg translator)
# These tests compare grep output with grg output

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURES="$SCRIPT_DIR/../fixtures"
GRG="${GRG:-grg}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0

# Increment helper (avoids set -e issues with arithmetic)
inc_passed() { PASSED=$((PASSED + 1)); }
inc_failed() { FAILED=$((FAILED + 1)); }

# Helper: extract just the matching lines (strip filenames, line numbers)
extract_matches() {
    # Remove filename: and line number prefixes, sort for comparison
    sed 's/^[^:]*://' | sed 's/^[0-9]*://' | sort
}

# Helper: run a test case
# Args: test_name grep_args...
run_test() {
    local name="$1"
    shift
    
    # Run grep and grg, extract just the matching content
    local grep_out rg_out
    grep_out=$(grep "$@" 2>/dev/null | extract_matches) || true
    grg_out=$($GRG "$@" 2>/dev/null | extract_matches) || true
    
    if [ "$grep_out" = "$grg_out" ]; then
        echo -e "${GREEN}✓${NC} $name"
        inc_passed
    else
        echo -e "${RED}✗${NC} $name"
        echo "  grep output:"
        echo "$grep_out" | sed 's/^/    /'
        echo "  grg output:"
        echo "$grg_out" | sed 's/^/    /'
        inc_failed
    fi
}

echo "Running grg acceptance tests..."
echo "Fixtures: $FIXTURES"
echo ""

# Basic pattern matching
run_test "basic pattern" "hello" "$FIXTURES/sample.txt"
run_test "case insensitive (-i)" "-i" "hello" "$FIXTURES/sample.txt"
run_test "line numbers (-n)" "-n" "foo" "$FIXTURES/sample.txt"
run_test "invert match (-v)" "-v" "hello" "$FIXTURES/sample.txt"
run_test "word boundary (-w)" "-w" "foo" "$FIXTURES/sample.txt"
run_test "recursive (-r)" "-r" "hello" "$FIXTURES"
run_test "recursive + case insensitive" "-ri" "hello" "$FIXTURES"

# Context lines
run_test "after context (-A)" "-A" "1" "foo" "$FIXTURES/sample.txt"
run_test "before context (-B)" "-B" "1" "foo" "$FIXTURES/sample.txt"
run_test "context both (-C)" "-C" "1" "foo" "$FIXTURES/sample.txt"

# Count and files
run_test "count matches (-c)" "-c" "hello" "$FIXTURES/sample.txt"
run_test "files with matches (-l)" "-l" "-r" "hello" "$FIXTURES"
run_test "only matching (-o)" "-o" "foo" "$FIXTURES/sample.txt"

# Fixed strings
run_test "fixed strings (-F)" "-F" "foo bar" "$FIXTURES/sample.txt"

# Extended regex
run_test "extended regex (-E)" "-E" "foo|baz" "$FIXTURES/sample.txt"

# Include/exclude patterns
run_test "include pattern" "-r" "--include=*.txt" "hello" "$FIXTURES"
run_test "include .py files" "-r" "--include=*.py" "hello" "$FIXTURES"

# Multiple patterns with -e
run_test "multiple patterns (-e)" "-e" "hello" "-e" "foo" "$FIXTURES/sample.txt"

# Long options
run_test "long --ignore-case" "--ignore-case" "hello" "$FIXTURES/sample.txt"
run_test "long --recursive" "--recursive" "hello" "$FIXTURES"
run_test "long --word-regexp" "--word-regexp" "foo" "$FIXTURES/sample.txt"

echo ""
echo "Results: $PASSED passed, $FAILED failed"

if [ $FAILED -gt 0 ]; then
    exit 1
fi
