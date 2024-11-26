#!/usr/bin/env bash
set -e

sudo apt-get install heaptrack

NARGO="nargo"


# Tests to be profiled for memory report
tests_to_profile=("keccak256" "workspace" "regression_4709")

current_dir=$(pwd)
execution_success_path="$current_dir/execution_success"
test_dirs=$(ls $execution_success_path)

FIRST="1"

echo "{\"memory_reports\": [ " > memory_report.json


for test_name in ${tests_to_profile[@]}; do    
        full_path=$execution_success_path"/"$test_name
        cd $full_path

        if [ $FIRST = "1" ]
        then
            FIRST="0"
        else
            echo " ," >> $current_dir"/memory_report.json"
        fi
        heaptrack --output $current_dir/$test_name"_heap" $NARGO compile --force
        heaptrack --analyze $current_dir/$test_name"_heap.gz" > $current_dir/$test_name"_heap_analysis.txt"
        rm $current_dir/$test_name"_heap.gz"
        consumption="$(grep 'peak heap memory consumption'  $current_dir/$test_name'_heap_analysis.txt')"
        len=${#consumption}-30
        peak=${consumption:30:len}
        rm $current_dir/$test_name"_heap_analysis.txt"
        echo -e " {\n    \"artifact_name\":\"$test_name\",\n    \"peak_memory\":\"$peak\"\n }" >> $current_dir"/memory_report.json"
done

echo "]}" >> $current_dir"/memory_report.json"

