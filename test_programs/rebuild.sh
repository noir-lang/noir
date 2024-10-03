#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status, treat unset variables as an error, and print commands as they are executed
set -e

process_dir() {
    local dir=$1
    local current_dir=$2
    local dir_name=$(basename "$dir")

    {
        echo "Processing $dir"

        if [[ ! -f "$dir/Nargo.toml" ]]; then
            echo "No Nargo.toml found in $dir. Removing directory."
            rm -rf "$dir"
            echo "$dir: skipped (no Nargo.toml)"
            return 0
        fi

        if [[ ! -d "$current_dir/acir_artifacts/$dir_name" ]]; then
            mkdir -p "$current_dir/acir_artifacts/$dir_name"
        fi

        cd "$dir"
        if [ -d ./target/ ]; then
            rm -r ./target/
        fi

        if ! nargo execute witness; then
            echo "$dir failed"
        else
            if [ -d "$current_dir/acir_artifacts/$dir_name/target" ]; then
                rm -r "$current_dir/acir_artifacts/$dir_name/target"
            fi
            mkdir "$current_dir/acir_artifacts/$dir_name/target"
            mkdir "$current_dir/acir_artifacts/$dir_name/proofs"

            mv ./target/$dir_name.json "$current_dir/acir_artifacts/$dir_name/target/program.json"
            mv ./target/*.gz "$current_dir/acir_artifacts/$dir_name/target/"
            echo "$dir succeeded"
        fi

        cd "$current_dir"
    } >> "$current_dir/rebuild.log" 2>&1
}

export -f process_dir

excluded_dirs=("workspace" "workspace_default_member")
current_dir=$(pwd)
base_path="$current_dir/execution_success"
dirs_to_process=()

# Remove existing artifacts and create a new directory
rm -rf "$current_dir/acir_artifacts"
mkdir -p "$current_dir/acir_artifacts"

# Gather directories to process, either from arguments or by default.
if [ $# -gt 0 ]; then
    for dir in "$@"; do
        dirs_to_process+=("$base_path/$dir")
    done
else
    for dir in $base_path/*; do
        if [[ ! -d $dir ]] || [[ " ${excluded_dirs[@]} " =~ " $(basename "$dir") " ]]; then
            continue
        fi
        dirs_to_process+=("$dir")
    done

    for dir in $current_dir/benchmarks/*; do
        if [[ ! -d $dir ]]; then
            continue
        fi
        dirs_to_process+=("$dir")
    done
fi

# Clear any existing rebuild.log
rm -f "$current_dir/rebuild.log"

# Process directories in parallel
parallel -j7  process_dir {} "$current_dir" ::: ${dirs_to_process[@]}

# Check rebuild.log for failures
if [ -f "$current_dir/rebuild.log" ]; then
    failed_dirs=($(grep -a 'failed' "$current_dir/rebuild.log" | awk '{print $1}'))
else
    echo "rebuild.log not found or empty. Check for errors." >&2
    exit 1
fi

# Print final status message after processing all directories
if [ ${#failed_dirs[@]} -ne 0 ]; then
    echo "Rebuild failed for the following directories:"
    for dir in "${failed_dirs[@]}"; do
        echo "- $dir"
    done
    exit 1
else
    echo "Rebuild Succeeded!"
fi
