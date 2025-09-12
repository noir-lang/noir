#!/usr/bin/env bash
set -e

NARGO=${NARGO:-nargo}

# These tests are incompatible with gas reporting
excluded_dirs=("workspace" "overlapping_dep_and_mod" "overlapping_dep_and_mod_fix" "workspace_default_member" "workspace_reexport_bug")

current_dir=$(pwd)

# We generate a Noir workspace which contains all of the test cases
# This allows us to generate a gates report using `nargo info` for all of them at once.


function collect_dirs {
  test_dirs=$(ls $current_dir/$1)

  for dir in $test_dirs; do
    # TODO(https://github.com/noir-lang/noir/issues/7835): example blocking issue
    # skip generated tests
    if [[ "${dir}" =~ ^(noirc_frontend_*|noirc_evaluator_*) ]]; then
      continue
    fi

    if [[ " ${excluded_dirs[@]} " =~ " ${dir} " ]]; then
      continue
    fi

    if [[ ! -f "$current_dir/$1/$dir/Nargo.toml" ]]; then
      echo "No Nargo.toml found in $dir. Removing directory."
      rm -rf "$current_dir/$1/$dir"
      echo "$dir: skipped (no Nargo.toml)"
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
collect_dirs fuzzing_failure

echo "]" >> Nargo.toml

if [ "$1" == "check" ]; then
  nargo fmt --check
else
  nargo fmt
fi


rm Nargo.toml
