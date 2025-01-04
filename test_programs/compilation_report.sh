#!/usr/bin/env bash
set -e

current_dir=$(pwd)
base_path="$current_dir/execution_success"

# Tests to be profiled for compilation report
tests_to_profile=("sha256_regression" "regression_4709" "ram_blowup_regression")

echo "{\"compilation_reports\": [ " > $current_dir/compilation_report.json

# If there is an argument that means we want to generate a report for only the current directory
if [ "$1" == "1" ]; then
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

    cd $base_path/$dir

    # The default package to run is the supplied list hardcoded at the top of the script
    PACKAGE_NAME=$dir
    # Otherwise default to the current directory as the package we want to run
    if [ "$1" == "1" ]; then
      PACKAGE_NAME=$(basename $current_dir)
    fi

    NUM_RUNS=1
    TOTAL_TIME=0

    # Passing a second argument will take an average of five runs
    # rather than 
    if [ "$2" == "1" ]; then
      NUM_RUNS=5
    fi

    for ((i = 1; i <= NUM_RUNS; i++)); do
      COMPILE_TIME=$((time nargo compile --force --silence-warnings) 2>&1 | grep real | grep -oE '[0-9]+m[0-9]+.[0-9]+s')
      # Convert time to seconds and add to total time
      TOTAL_TIME=$(echo "$TOTAL_TIME + $(echo $COMPILE_TIME | sed -E 's/([0-9]+)m([0-9.]+)s/\1 * 60 + \2/' | bc)" | bc)
    done

    AVG_TIME=$(echo "$TOTAL_TIME / $NUM_RUNS" | bc -l)
    # Keep only last three decimal points
    AVG_TIME=$(awk '{printf "%.3f\n", $1}' <<< "$AVG_TIME")

    echo -e " {\n    \"artifact_name\":\"$PACKAGE_NAME\",\n    \"time\":\""$AVG_TIME"s\"" >> $current_dir/compilation_report.json
    
    if (($ITER == $NUM_ARTIFACTS)); then
        echo "}" >> $current_dir/compilation_report.json
    else 
        echo "}, " >> $current_dir/compilation_report.json
    fi

    ITER=$(( $ITER + 1 ))
done

echo "]}" >> $current_dir/compilation_report.json
