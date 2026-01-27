#!/usr/bin/env bash
set -eo pipefail

# These tests are incompatible with artifact size reporting
excluded_dirs=(
  "workspace"
  "workspace_default_member"
  "double_verify_nested_proof"
  "overlapping_dep_and_mod"
  "comptime_println"
  # This test utilizes enums which are experimental
  "regression_7323"
)

current_dir=$(pwd)
base_path="$current_dir/execution_success"
test_dirs=$(ls $base_path)

# We generate a Noir workspace which contains all of the test cases
# This allows us to generate a gates report using `nargo info` for all of them at once.

echo "[workspace]" > Nargo.toml
echo "members = [" >> Nargo.toml

for dir in $test_dirs; do
    if [[ " ${excluded_dirs[@]} " =~ " ${dir} " ]]; then
      continue
    fi

    if [[ ${CI-false} = "true" ]] && [[ " ${ci_excluded_dirs[@]} " =~ " ${dir} " ]]; then
      continue
    fi

    echo "  \"execution_success/$dir\"," >> Nargo.toml
done

echo "]" >> Nargo.toml

nargo info --silence-warnings --force-brillig --json --inliner-aggressiveness $1 | jq -r ".programs[].functions = []"  > gates_report_brillig.json

rm Nargo.toml
