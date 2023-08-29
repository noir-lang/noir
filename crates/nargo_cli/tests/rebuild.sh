#!/bin/bash
set -e

excluded_dirs=("workspace" "workspace_default_member")

cd ./execution_success

# Loop over every directory
for dir in ./*; do
  if [[ ! -d $dir ]]; then
    continue
  fi

  dir_name=$(basename "$dir")
  if [[ ! " ${excluded_dirs[@]} " =~ " ${dir_name} " ]]; then
      cd $dir
      if [ -d ./target/ ]; then
        rm -r ./target/
      fi
      nargo compile && nargo execute witness

      # Extract bytecode field from JSON, base64 decode it, and save it to a target directory
      if [ -f ./target/${dir_name}.json ]; then
          jq -r '.bytecode' ./target/${dir_name}.json | base64 -d > ./target/${dir_name}.bytecode
      fi

      # Delete the JSON file after extracting bytecode field
      rm ./target/${dir_name}.json

      cd ..
  fi
done
