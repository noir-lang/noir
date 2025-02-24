#!/usr/bin/env bash
set -e

# These tests are incompatible with gas reporting
excluded_dirs=("workspace" "overlapping_dep_and_mod" "overlapping_dep_and_mod_fix" "workspace_default_member" "workspace_reexport_bug")

# These tests cause failures in CI with a stack overflow for some reason.
ci_excluded_dirs=("eddsa")

current_dir=$(pwd)

# We generate a Noir workspace which contains all of the test cases
# This allows us to generate a gates report using `nargo info` for all of them at once.


function collect_dirs {
  test_dirs=$(ls $current_dir/$1)

  for dir in $test_dirs; do
    if [[ " ${excluded_dirs[@]} " =~ " ${dir} " ]]; then
      continue
    fi

    if [[ ${CI-false} = "true" ]] && [[ " ${ci_excluded_dirs[@]} " =~ " ${dir} " ]]; then
      continue
    fi

    echo "  \"$1/$dir\"," >> Nargo.toml
done
}

echo "[workspace]" > Nargo.toml
echo "members = [" >> Nargo.toml

collect_dirs compile_success_empty
collect_dirs compile_success_contract
collect_dirs compile_success_no_bug
collect_dirs compile_success_with_bug
collect_dirs execution_success
collect_dirs noir_test_success

echo "]" >> Nargo.toml

if [ "$1" == "check" ]; then
  nargo fmt --check
else
  nargo fmt
fi


rm Nargo.toml
