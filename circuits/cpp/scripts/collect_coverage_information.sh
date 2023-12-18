#!/usr/bin/env bash

# Check that the correct number of args have been provided
if [ $# -ne 2 ]; then
    echo "Usage: $0 <llvm-profdata command> <llvm-cov command>"
    exit 1
fi

# Check that the llvm-profdata command exists
llvm_profdata_command="$1"
if ! command -v "$llvm_profdata_command" >/dev/null; then
    echo "$llvm_profdata_command could not be found"
    exit 1
fi

# Check that the llvm-cov command exists
llvm_cov_command="$2"
if ! command -v "$llvm_cov_command" >/dev/null; then
    echo "$llvm_cov_command could not be found"
    exit 1
fi

# Check for existence of test binaries
WORKING_DIRECTORY=$(pwd)
if [ ! -d "$WORKING_DIRECTORY/bin" ]; then
    echo "No binary directory. Are you sure that you are in a build directory and you've compiled binaries?"
    exit 1
fi

# Check for existence of profdata files, run make sure tests have been run and compiled with Debug and coverage flags enabled
if [ ! -d "$WORKING_DIRECTORY/bin/profdata" ]; then
    echo "No profdata directory. Have you run any tests with profiling?"
    exit 1
fi

# Find non empty profiles and store them in an array
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

# If there is one profile, output it into its coverate report into its own folder
if [ ${#non_empty_profiles[@]} -eq 1 ]; then
    mkdir -p "$WORKING_DIRECTORY/merged_profdata/"
    rm -f "$WORKING_DIRECTORY/merged_profdata/default.profdata"
    $llvm_profdata_command merge -sparse "$WORKING_DIRECTORY/bin/profdata/${!non_empty_profiles[@]}."*.profraw -o "$WORKING_DIRECTORY/merged_profdata/default.profdata"
    rm -rf "$WORKING_DIRECTORY/${non_empty_profiles[0]}_coverage_report"
    mkdir "$WORKING_DIRECTORY/${non_empty_profiles[0]}_coverage_report"
    $llvm_cov_command show -output-dir="$WORKING_DIRECTORY/${!non_empty_profiles[@]}_coverage_report" -format=html "$WORKING_DIRECTORY/bin/${!non_empty_profiles[@]}_tests" -instr-profile="$WORKING_DIRECTORY/merged_profdata/default.profdata" -ignore-filename-regex=".*_deps.*"
fi

# If there are many reports, output all of the coverage reports into one folder
if [ ${#non_empty_profiles[@]} -gt 1 ]; then
    mkdir -p "$WORKING_DIRECTORY/merged_profdata/"
    rm -f "$WORKING_DIRECTORY/merged_profdata/default.profdata"

    # Merge related profdata files into one file named default.profdata
    $llvm_profdata_command merge -sparse "$WORKING_DIRECTORY/bin/profdata/"*.profraw -o "$WORKING_DIRECTORY/merged_profdata/default.profdata"
    additional_objects=""
    for non_empty_profile_base in "${!non_empty_profiles[@]}"; do
        additional_objects+="-object  $WORKING_DIRECTORY/bin/${non_empty_profile_base}_tests "
    done
    object_string=${additional_objects#"-object"}

    # Output the coverage report into `all_tests_coverage_report` folder
    rm -rf "$WORKING_DIRECTORY/all_tests_coverage_report"
    mkdir "$WORKING_DIRECTORY/all_tests_coverage_report"
    $llvm_cov_command show -output-dir="$WORKING_DIRECTORY/all_tests_coverage_report" -format=html $object_string -instr-profile="$WORKING_DIRECTORY/merged_profdata/default.profdata" -ignore-filename-regex=".*_deps.*"
fi