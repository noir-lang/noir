#!/bin/bash
set -e

excluded_dirs=("workspace" "workspace_default_member")

current_dir=$(pwd)
base_path="$current_dir/execution_success"

# Ensure the base acir_artifacts directory exists
mkdir -p $current_dir/acir_artifacts

# Loop over every directory
for dir in $base_path/*; do
  if [[ ! -d $dir ]]; then
    continue
  fi

  dir_name=$(basename "$dir")

  if [[ ! " ${excluded_dirs[@]} " =~ " ${dir_name} " ]]; then
      echo "Creating directory $current_dir/acir_artifacts/$dir_name"
      mkdir -p $current_dir/acir_artifacts/$dir_name
      echo "Directory created."

      cd $dir
      if [ -d ./target/ ]; then
        rm -r ./target/
      fi
      nargo compile && nargo execute witness

      # Extract bytecode field from JSON, base64 decode it, and save it to the target directory
      if [ -f ./target/${dir_name}.json ]; then
          jq -r '.bytecode' ./target/${dir_name}.json | base64 -d > ./target/${dir_name}.bytecode
      fi

      # Delete the JSON file after extracting bytecode field
      rm ./target/${dir_name}.json
      
      # Move the target directory to the corresponding directory in acir_artifacts
      mv ./target/ $current_dir/acir_artifacts/$dir_name/

      cd $base_path
  fi
done
