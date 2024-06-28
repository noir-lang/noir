#!/usr/bin/env bash
set -e

NO_PARALLEL=${1:-}

process_dir() {
    local dir=$1
    local current_dir=$2
    local dir_name=$(basename "$dir")

    if [[ ! -f "$dir/Nargo.toml" ]]; then
      # This directory isn't a proper test but just hold some stale build artifacts
      # We then delete it and carry on.
      rm -rf $dir
      return 0
    fi


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

for dir in $current_dir/benchmarks/*; do
    if [[ ! -d $dir ]]; then
        continue
    fi
    dirs_to_process+=("$dir")
done

pids=() # Array to hold PIDs of background processes
dirs_map=() # Array to map PIDs to directories

if [ -z $NO_PARALLEL ]; then
    # Process directories in parallel
    for dir in "${dirs_to_process[@]}"; do
        process_dir "$dir" "$current_dir" &  # Run process_dir in the background
        pid=$!  # Get PID of the last background command
        pids+=($pid)  # Add PID to the pids array
        dirs_map[$pid]=$dir  # Map PID to the directory being processed
    done
else
    # Process directories sequentially
    for dir in "${dirs_to_process[@]}"; do
        process_dir "$dir" "$current_dir"  # Run process_dir in the foreground
        pid=$!  # Get PID of the last command
        pids+=($pid)  # Add PID to the pids array
        dirs_map[$pid]=$dir  # Map PID to the directory being processed
    done
fi

# Store the failed processes
failed_pids=()
# Check the exit status of each background job.
for pid in "${pids[@]}"; do
    if ! wait $pid; then  # Wait for the process to complete, check if it failed
        exit_status=$?  # Capture the failed exit status
        failed_pids+=($pid)
    fi
done

echo ""

# Exit with a failure status if any job failed.
if [ ! -z "$exit_status" ]; then
    echo "Rebuild failed for directories:"
    # Print the failed directories after waiting for each process to complete
    for pid in "${failed_pids[@]}"; do
        echo "${dirs_map[$pid]}"
    done
    exit $exit_status
fi
echo "Rebuild Succeeded!"