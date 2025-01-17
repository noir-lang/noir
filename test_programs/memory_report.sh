#!/usr/bin/env bash
set -e

sudo apt-get install heaptrack

NARGO="nargo"
PARSE_MEMORY=$(realpath "$(dirname "$0")/parse_memory.sh")


# Tests to be profiled for memory report
tests_to_profile=("keccak256" "workspace" "regression_4709" "ram_blowup_regression")

current_dir=$(pwd)
base_path="$current_dir/execution_success"

# If there is an argument that means we want to generate a report for only the current directory
if [ "$1" == "1" ]; then
  base_path="$current_dir"
  tests_to_profile=(".")
fi

FIRST="1"

FLAGS=${FLAGS:- ""}
echo "{\"memory_reports\": [ " > memory_report.json

for test_name in ${tests_to_profile[@]}; do    
        cd $base_path/$test_name

        if [ $FIRST = "1" ]
        then
            FIRST="0"
        else
            echo " ," >> $current_dir"/memory_report.json"
        fi

        if [ "$1" == "1" ]; then
            test_name=$(basename $current_dir)
        fi

        COMMAND="compile --force --silence-warnings $FLAGS"
        if [ "$2" == "1" ]; then
            COMMAND="execute --silence-warnings"
        fi

        heaptrack --output $current_dir/$test_name"_heap" $NARGO $COMMAND 
        if test -f $current_dir/$test_name"_heap.gz"; 
        then 
            heaptrack --analyze $current_dir/$test_name"_heap.gz" > $current_dir/$test_name"_heap_analysis.txt"
            rm $current_dir/$test_name"_heap.gz"
        else 
            heaptrack --analyze $current_dir/$test_name"_heap.zst" > $current_dir/$test_name"_heap_analysis.txt"
            rm $current_dir/$test_name"_heap.zst"
        fi
        consumption="$(grep 'peak heap memory consumption'  $current_dir/$test_name'_heap_analysis.txt')"
        len=${#consumption}-30
        peak=${consumption:30:len}
        rm $current_dir/$test_name"_heap_analysis.txt"
        peak_memory=$($PARSE_MEMORY $peak)
        echo -e " {\n    \"artifact_name\":\"$test_name\",\n    \"peak_memory\":\"$peak_memory\"\n }" >> $current_dir"/memory_report.json"
done

echo "]}" >> $current_dir"/memory_report.json"

