#!/usr/bin/env bash
set -e

# These tests are incompatible with gas reporting
# excluded_dirs=(
#   "workspace" 
#   "workspace_default_member" 
#   "double_verify_nested_proof" 
#   "overlapping_dep_and_mod" 
#   "comptime_println"
#   #  Takes a very long time to execute as large loops do not get simplified.
#   "regression_4709"
#   #  bit sizes for bigint operation doesn't match up.
#   "bigint"
#   #  Expected to fail as test asserts on which runtime it is in.
#   "is_unconstrained"
# )

current_dir=$(pwd)
base_path="$current_dir/execution_success"
test_dirs=$(ls $base_path)

# Tests to be profiled for memory report
tests_to_profile=("keccak256" "workspace" "regression_4709" "ram_blowup_regression")
echo "{\"compilation_reports\": [ " > $current_dir/compilation_report.json

ITER="1"
NUM_ARTIFACTS=${#tests_to_profile[@]}

for dir in ${tests_to_profile[@]}; do 
    if [[ " ${excluded_dirs[@]} " =~ " ${dir} " ]]; then
      continue
    fi

    if [[ ${CI-false} = "true" ]] && [[ " ${ci_excluded_dirs[@]} " =~ " ${dir} " ]]; then
      continue
    fi

    cd $base_path/$dir

    COMPILE_TIME=$((time nargo compile --force) 2>&1 | grep real | grep -oE '[0-9]+m[0-9]+.[0-9]+s')
    echo -e " {\n    \"artifact_name\":\"$dir\",\n    \"time\":\"$COMPILE_TIME\"\n" >> $current_dir/compilation_report.json

    if (($ITER == $NUM_ARTIFACTS)); then
        echo "}" >> $current_dir/compilation_report.json
    else 
        echo "}, " >> $current_dir/compilation_report.json
    fi

    ITER=$(( $ITER + 1 ))
done

echo "]}" >> $current_dir/compilation_report.json

