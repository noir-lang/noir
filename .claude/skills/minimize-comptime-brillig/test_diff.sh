#!/bin/bash
# Test script for comptime_vs_brillig differences
# Usage: test_diff.sh [project_dir]
#   project_dir: Path to the Noir project (default: current directory)
# Exit code 0 = bug still present (outputs differ)
# Exit code 1 = bug not reproduced (outputs same)

PROJECT_DIR="${1:-.}"

if [ ! -f "$PROJECT_DIR/src/main.nr" ]; then
    echo "Error: $PROJECT_DIR/src/main.nr not found"
    echo "Usage: $0 [project_dir]"
    exit 2
fi

# Run nargo, capture stdout, discard stderr, filter nargo status messages
# Status messages look like: [project_name] Circuit witness successfully solved
run_nargo() {
    cargo run --release -p nargo_cli -- execute --program-dir "$PROJECT_DIR" --silence-warnings 2>/dev/null | grep -v "^\[.*\] .*solved\|^\[.*\] Witness saved"
}

# Test comptime version (expects comptime block in main)
echo "=== COMPTIME ==="
COMPTIME=$(run_nargo)
echo "$COMPTIME"

# Switch to non-comptime
sed -i 's/comptime {/{ \/\/ comptime disabled/' "$PROJECT_DIR/src/main.nr"

echo ""
echo "=== BRILLIG ==="
BRILLIG=$(run_nargo)
echo "$BRILLIG"

# Restore comptime
sed -i 's/{ \/\/ comptime disabled/comptime {/' "$PROJECT_DIR/src/main.nr"

echo ""
if [ "$COMPTIME" = "$BRILLIG" ]; then
    echo "SAME - bug not reproduced"
    exit 1
else
    echo "DIFFERENT - bug still present"
    exit 0
fi
