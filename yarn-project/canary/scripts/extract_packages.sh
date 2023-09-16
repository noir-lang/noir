#!/bin/bash

FILE=$1

# Capture the output of the jq command in a Bash array
mapfile -t TARGET_PKGS < <(jq -r '.dependencies | keys[] | select(startswith("@aztec/") and . != "@aztec/end-to-end")' $FILE)

# Loop through the array and print each element on a new line
for pkg in "${TARGET_PKGS[@]}"; do
  echo "$pkg"
done