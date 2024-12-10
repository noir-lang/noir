#!/usr/bin/env bash
set -e

current_dir=$(pwd)
base_path="$current_dir/execution_success"

# Tests to be profiled for compilation report
tests_to_profile=("sha256_regression" "regression_4709" "ram_blowup_regression")

echo "{\"compilation_reports\": [ " > $current_dir/compilation_report.json

# If there is an argument that means we want to re-use an already existing compilation report
# rather than generating a new one.
# When reusing a report, the script can only profile one additional test at the moment.
if [ "$#" -eq 0 ]; then
  # echo "{\"compilation_reports\": [ " > $current_dir/compilation_report.json
else 
  # Delete last two lines so that we can re-use the previous report 
  # sed -i '${/^$/d;}' compilation_report.json | sed -i '$d' compilation_report.json | sed -i '$d' compilation_report.json

  # echo "}, " >> compilation_report.json

  # The additional report is expected to be in the current directory
  base_path="$current_dir"
  tests_to_profile=(".")
fi

ITER="1"
NUM_ARTIFACTS=${#tests_to_profile[@]}

for dir in ${tests_to_profile[@]}; do 
    if [[ " ${excluded_dirs[@]} " =~ " ${dir} " ]]; then
      continue
    fi

    if [[ ${CI-false} = "true" ]] && [[ " ${ci_excluded_dirs[@]} " =~ " ${dir} " ]]; then
      continue
    fi

    echo $base_path/$dir

    PACKAGE_NAME=$dir
    if [ "$#" -ne 0 ]; then
      PACKAGE_NAME=$(basename $current_dir)
    fi

    echo $PACKAGE_NAME

    COMPILE_TIME=$((time nargo compile --force) 2>&1 | grep real | grep -oE '[0-9]+m[0-9]+.[0-9]+s')
    echo -e " {\n    \"artifact_name\":\"$PACKAGE_NAME\",\n    \"time\":\"$COMPILE_TIME\"" >> $current_dir/compilation_report.json
    
    if (($ITER == $NUM_ARTIFACTS)); then
        echo "}" >> $current_dir/compilation_report.json
    else 
        echo "}, " >> $current_dir/compilation_report.json
    fi

    ITER=$(( $ITER + 1 ))
done

echo "]}" >> $current_dir/compilation_report.json
