#!/bin/bash
set -e

# Parse exclude and fail directories from cargo.toml
exclude_dirs=$(grep "^exclude" config.toml | sed 's/exclude = \[//;s/\]//;s/\"//g;s/ //g' | tr ',' '\n')
fail_dirs=$(grep "^fail" config.toml | sed 's/fail = \[//;s/\]//;s/\"//g;s/ //g' | tr ',' '\n')

# Convert them to array
exclude_array=($exclude_dirs)
fail_array=($fail_dirs)

# Merge exclude and fail arrays
exclude_fail_dirs=("${exclude_array[@]}" "${fail_array[@]}" "workspace")

# Loop over every directory
for dir in ./*; do
  if [[ ! -d $dir ]]; then
    continue
  fi

  dir_name=$(basename "$dir")
  if [[ ! " ${exclude_fail_dirs[@]} " =~ " ${dir_name} " ]]; then
      cd $dir
      if [ -d ./target/ ]; then
        rm -r ./target/
      fi
      nargo compile && nargo execute witness
      cd ..
  fi
done

