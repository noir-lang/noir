#!/usr/bin/env bash
set -e

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
# dirs_to_process=()
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

parallel -j0  process_dir {} "$current_dir" ::: ${dirs_to_process[@]}

echo "Rebuild Succeeded!"
