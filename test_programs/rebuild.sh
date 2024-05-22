#!/usr/bin/env bash
set -e

NO_PARALLEL=${1:-}

process_dir() {
    local dir=$1
    local current_dir=$2
    local dir_name=$(basename "$dir")

    if [[ ! -d "$current_dir/acir_artifacts/$dir_name" ]]; then
      mkdir -p $current_dir/acir_artifacts/$dir_name
    fi

    cd $dir
    if [ -d ./target/ ]; then
      rm -r ./target/
    fi
    nargo execute witness

    if [ -d "$current_dir/acir_artifacts/$dir_name/target" ]; then
      rm -r "$current_dir/acir_artifacts/$dir_name/target"
    fi
    mkdir $current_dir/acir_artifacts/$dir_name/target

    mv ./target/$dir_name.json $current_dir/acir_artifacts/$dir_name/target/program.json
    mv ./target/*.gz $current_dir/acir_artifacts/$dir_name/target/

    cd $current_dir
}

export -f process_dir

excluded_dirs=("workspace" "workspace_default_member")
current_dir=$(pwd)
base_path="$current_dir/execution_success"

rm -rf $current_dir/acir_artifacts
mkdir -p $current_dir/acir_artifacts

# Gather directories to process.
dirs_to_process=()
for dir in $base_path/*; do
    if [[ ! -d $dir ]] || [[ " ${excluded_dirs[@]} " =~ " $(basename "$dir") " ]]; then
        continue
    fi
    dirs_to_process+=("$dir")
done

# Process each directory in parallel
pids=()
if [ -z $NO_PARALLEL ]; then
for dir in "${dirs_to_process[@]}"; do
    process_dir "$dir" "$current_dir" &
    pids+=($!)
done
else
for dir in "${dirs_to_process[@]}"; do
    process_dir "$dir" "$current_dir"
    pids+=($!)
done
fi

# Check the exit status of each background job.
for pid in "${pids[@]}"; do
    wait $pid || exit_status=$?
done

# Exit with a failure status if any job failed.
if [ ! -z "$exit_status" ]; then
    echo "Rebuild failed!"
    exit $exit_status
fi
echo "Rebuild Succeeded!"
