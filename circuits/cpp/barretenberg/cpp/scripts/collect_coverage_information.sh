#!/bin/bash

if [ $# -ne 2 ]; then
    echo "Usage: $0 <llvm-profdata command> <llvm-cov command>"
    exit 1
fi

llvm_profdata_command="$1"
if ! command -v "$llvm_profdata_command" >/dev/null; then
    echo "$llvm_profdata_command could not be found"
    exit 1
fi

llvm_cov_command="$2"
if ! command -v "$llvm_cov_command" >/dev/null; then
    echo "$llvm_cov_command could not be found"
    exit 1
fi
WORKING_DIRECTORY=$(pwd)
if [ ! -d "$WORKING_DIRECTORY/bin" ]; then
    echo "No binary directory. Are you sure that you are in a build directory and you've compiled binaries?"
    exit 1
fi

if [ ! -d "$WORKING_DIRECTORY/bin/profdata" ]; then
    echo "No profdata directory. Have you run any tests with profiling?"
    exit 1
fi

declare -A non_empty_profiles
for profdata_file in $(ls "$WORKING_DIRECTORY/bin/profdata"); do
    if [ -s "$WORKING_DIRECTORY/bin/profdata/$profdata_file" ]; then
        non_empty_profiles["${profdata_file%%.*}"]=1
    fi
done
if [ ${#non_empty_profiles[@]} -eq 0 ]; then
    echo "No profiles found"
    exit 1
fi

if [ ${#non_empty_profiles[@]} -eq 1 ]; then
    mkdir -p "$WORKING_DIRECTORY/merged_profdata/"
    rm -f "$WORKING_DIRECTORY/merged_profdata/default.profdata"
    $llvm_profdata_command merge -sparse "$WORKING_DIRECTORY/bin/profdata/${!non_empty_profiles[@]}."*.profraw -o "$WORKING_DIRECTORY/merged_profdata/default.profdata"
    rm -rf "$WORKING_DIRECTORY/${non_empty_profiles[0]}_coverage_report"
    mkdir "$WORKING_DIRECTORY/${non_empty_profiles[0]}_coverage_report"
    $llvm_cov_command show -output-dir="$WORKING_DIRECTORY/${!non_empty_profiles[@]}_coverage_report" -format=html "$WORKING_DIRECTORY/bin/${!non_empty_profiles[@]}_tests" -instr-profile="$WORKING_DIRECTORY/merged_profdata/default.profdata" -ignore-filename-regex=".*_deps.*"
fi

if [ ${#non_empty_profiles[@]} -gt 1 ]; then
    mkdir -p "$WORKING_DIRECTORY/merged_profdata/"
    rm -f "$WORKING_DIRECTORY/merged_profdata/default.profdata"
    $llvm_profdata_command merge -sparse "$WORKING_DIRECTORY/bin/profdata/"*.profraw -o "$WORKING_DIRECTORY/merged_profdata/default.profdata"
    additional_objects=""
    for non_empty_profile_base in "${!non_empty_profiles[@]}"; do
        additional_objects+="-object  $WORKING_DIRECTORY/bin/${non_empty_profile_base}_tests "
    done
    object_string=${additional_objects#"-object"}
    rm -rf "$WORKING_DIRECTORY/all_tests_coverage_report"
    mkdir "$WORKING_DIRECTORY/all_tests_coverage_report"
    $llvm_cov_command show -output-dir="$WORKING_DIRECTORY/all_tests_coverage_report" -format=html $object_string -instr-profile="$WORKING_DIRECTORY/merged_profdata/default.profdata" -ignore-filename-regex=".*_deps.*"
fi
