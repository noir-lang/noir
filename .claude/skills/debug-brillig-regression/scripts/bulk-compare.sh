#!/usr/bin/env bash
# Run the Brillig gates report with two nargo binaries and save the JSON reports.
# Usage: bulk-compare.sh <nargo_base> <nargo_current> <inliner> <work_dir>
#
# Runs gates_report_brillig.sh from the test_programs directory using each
# nargo binary, saving the results to <work_dir>/report_base.json and
# <work_dir>/report_current.json.
#
# Outputs:
#   <work_dir>/report_base.json
#   <work_dir>/report_current.json

set -euo pipefail

if [[ $# -lt 4 ]]; then
    echo "Usage: $0 <nargo_base> <nargo_current> <inliner> <work_dir>" >&2
    exit 1
fi

nargo_base="$(realpath "$1")"
nargo_current="$(realpath "$2")"
inliner="$3"
work_dir="$4"
repo_root="$(git rev-parse --show-toplevel)"

mkdir -p "$work_dir"

cd "$repo_root/test_programs"

PATH_BAK="$PATH"

# Run with base nargo
echo "Running gates report with base nargo..."
export PATH="$(dirname "$nargo_base"):$PATH"
ln -sf "$nargo_base" "$(dirname "$nargo_base")/nargo"
./gates_report_brillig.sh "$inliner"
mv gates_report_brillig.json "$work_dir/report_base.json"

# Run with current nargo
echo "Running gates report with current nargo..."
export PATH="$(dirname "$nargo_current"):$PATH"
ln -sf "$nargo_current" "$(dirname "$nargo_current")/nargo"
./gates_report_brillig.sh "$inliner"
mv gates_report_brillig.json "$work_dir/report_current.json"

export PATH="$PATH_BAK"
echo "Reports saved to $work_dir/report_base.json and $work_dir/report_current.json"
