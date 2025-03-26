#!/usr/bin/env bash
set -e

PARSE_TIME=$(realpath "$(dirname "$0")/parse_time.sh")
current_dir=$(pwd)
base_path="$current_dir/execution_success"

# Tests to be profiled for compilation report
tests_to_profile=("regression_4709" "ram_blowup_regression" "global_var_regression_entry_points")

echo "[ " > $current_dir/compilation_report.json

# If there is an argument that means we want to generate a report for only the current directory
if [ "$1" == "1" ]; then
  base_path="$current_dir"
  tests_to_profile=(".")
fi

ITER="1"
NUM_ARTIFACTS=${#tests_to_profile[@]}
FLAGS=${FLAGS:- ""}

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

    NUM_RUNS=$2
    TOTAL_TIME=0

    for ((i = 1; i <= NUM_RUNS; i++)); do
      NOIR_LOG=trace NARGO_LOG_DIR=./tmp nargo compile --force --silence-warnings $FLAGS
    done

    TIMES=($(jq -r '. | select(.target == "nargo::cli" and .fields.message == "close") | .fields."time.busy"' ./tmp/*))

    AVG_TIME=$(awk -v RS=" " -v parse_time="$PARSE_TIME"  '
        {
            # Times are formatted annoyingly so we need to parse it.
            parse_time" "$1 | getline current_time
            close(parse_time" "$1)
            sum += current_time;
            n++;
        }
        END {   
            if (n > 0)
                printf "%.3f\n", sum / n
            else
                printf "%.3f\n", 0
        }' <<<"${TIMES[@]}")

    jq -rc "{name: \"$PACKAGE_NAME\", value: \""$AVG_TIME"\" | tonumber, unit: \"s\"}" --null-input >> $current_dir/compilation_report.json

    if (($ITER != $NUM_ARTIFACTS)); then
        echo "," >> $current_dir/compilation_report.json
    fi

    rm -rf ./tmp

    ITER=$(( $ITER + 1 ))
done

echo "]" >> $current_dir/compilation_report.json
