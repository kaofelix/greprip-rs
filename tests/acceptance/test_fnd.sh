#!/usr/bin/env bash
# Acceptance tests for fnd (find → fd translator)
# These tests compare find output with fnd output

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURES="$SCRIPT_DIR/../fixtures"
FND="${FND:-fnd}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0

# Increment helper (avoids set -e issues with arithmetic)
inc_passed() { PASSED=$((PASSED + 1)); }
inc_failed() { FAILED=$((FAILED + 1)); }

# Helper: normalize output (sort, strip trailing slashes for dirs)
# Also filters out the search root itself (find includes it, fd doesn't)
normalize() {
    sed 's|/$||' | sort
}

# Normalize find output - remove the exact search path if it appears as a standalone line
normalize_find() {
    local search_path="$1"
    normalize | grep -v "^${search_path}$" || true
}

# Helper: run a test case
# Args: test_name find_args...
run_test() {
    local name="$1"
    shift
    
    # First arg is typically the search path
    local search_path="$1"
    # Resolve to absolute path for filtering
    local abs_search_path
    abs_search_path=$(cd "$search_path" 2>/dev/null && pwd) || abs_search_path="$search_path"
    
    # Run find and fnd, normalize for comparison
    # Filter out the search root from find output (find includes it, fd doesn't)
    local find_out fnd_out
    find_out=$(find "$@" 2>/dev/null | normalize | grep -v "^${abs_search_path}$" | grep -v "^${search_path}$") || true
    fnd_out=$($FND "$@" 2>/dev/null | normalize) || true
    
    if [ "$find_out" = "$fnd_out" ]; then
        echo -e "${GREEN}✓${NC} $name"
        inc_passed
    else
        echo -e "${RED}✗${NC} $name"
        echo "  find output:"
        echo "$find_out" | sed 's/^/    /'
        echo "  fnd output:"
        echo "$fnd_out" | sed 's/^/    /'
        inc_failed
    fi
}

echo "Running fnd acceptance tests..."
echo "Fixtures: $FIXTURES"
echo ""

# Basic usage
run_test "list all files" "$FIXTURES"
run_test "find by name" "$FIXTURES" "-name" "*.txt"
run_test "find by name (case insensitive)" "$FIXTURES" "-iname" "*.TXT"
run_test "find files only" "$FIXTURES" "-type" "f"
run_test "find directories only" "$FIXTURES" "-type" "d"

# Depth control
run_test "max depth" "$FIXTURES" "-maxdepth" "1"
run_test "min depth" "$FIXTURES" "-mindepth" "1"

# Combinations
run_test "name + type file" "$FIXTURES" "-name" "*.py" "-type" "f"
run_test "type + maxdepth" "$FIXTURES" "-type" "f" "-maxdepth" "1"

# Exec tests
run_test "exec basename" "$FIXTURES" "-name" "*.txt" "-exec" "basename" "{}" ";"
run_test "exec with type" "$FIXTURES" "-type" "f" "-exec" "basename" "{}" ";"

echo ""
echo "Results: $PASSED passed, $FAILED failed"

if [ $FAILED -gt 0 ]; then
    exit 1
fi
