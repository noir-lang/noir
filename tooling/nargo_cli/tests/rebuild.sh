#!/bin/bash
set -e

excluded_dirs=("workspace" "workspace_default_member")

current_dir=$(pwd)
base_path="$current_dir/execution_success"

# Clear the acir_artifacts directory of any existing artifacts
rm -rf $current_dir/acir_artifacts
mkdir -p $current_dir/acir_artifacts

# Loop over every directory
for dir in $base_path/*; do
  if [[ ! -d $dir ]]; then
    continue
  fi

  dir_name=$(basename "$dir")

  if [[ ! " ${excluded_dirs[@]} " =~ " ${dir_name} " ]]; then
      if [[ ! -d "$current_dir/acir_artifacts/$dir_name" ]]; then
        mkdir -p $current_dir/acir_artifacts/$dir_name
      fi

      cd $dir
      if [ -d ./target/ ]; then
        rm -r ./target/
      fi
      nargo compile && nargo execute witness

      # Rename witness.tr to witness.gz
      if [ -f ./target/witness.tr ]; then
        mv ./target/witness.tr ./target/witness.gz
      fi

      # Extract bytecode field from JSON, base64 decode it, and save it to the target directory
      if [ -f ./target/${dir_name}.json ]; then
          jq -r '.bytecode' ./target/${dir_name}.json | base64 -d > ./target/acir.gz
      fi

      # Delete the JSON file after extracting bytecode field
      rm ./target/${dir_name}.json

      # Clear the target directory in acir_artifacts
      if [ -d "$current_dir/acir_artifacts/$dir_name/target" ]; then
        rm -r "$current_dir/acir_artifacts/$dir_name/target"
      fi
      mkdir $current_dir/acir_artifacts/$dir_name/target
      
      # Move the artifacts from the target directory to the corresponding directory in acir_artifacts
      mv ./target/*.gz $current_dir/acir_artifacts/$dir_name/target/

      cd $base_path
  fi
done
