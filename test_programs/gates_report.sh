#!/usr/bin/env bash
set -e

# These tests are incompatible with gas reporting
excluded_dirs=("workspace" "workspace_default_member" "double_verify_nested_proof")

# These tests cause failures in CI with a stack overflow for some reason.
ci_excluded_dirs=("eddsa")

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

nargo info --json > gates_report.json

rm Nargo.toml
