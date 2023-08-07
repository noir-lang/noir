#!/bin/bash
set -e

excluded_dirs=("workspace")

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
      cd ..
  fi
done

