#!/bin/bash

# Define old and new scopes
old_scope="@noir-lang"
new_scope="@kevaundray"

# Read workspace configuration from package.json
# using jq to parse JSON, so make sure jq is installed on your system
workspaces=( $(jq -r '.workspaces[]' package.json) )

# Iterate over each workspace directory
for workspace in "${workspaces[@]}"; do
  # Check if workspace is a directory
  if [ -d "$workspace" ]; then
    # Find all package.json files within the workspace directory and its subdirectories
    find "$workspace" -name 'package.json' -print0 | while read -r -d $'\0' package_file; do
      # Change scope in package.json file
      sed -i'' -e "s/$old_scope/$new_scope/g" "$package_file"
      
      # Determine the directory of the package.json file
      dir=$(dirname "$package_file")
      
      # Update references in all .js, .ts, and .json files within the package directory
      find "$dir" -type f \( -name "*.js" -o -name "*.ts" -o -name "*.json" \) -exec sed -i'' -e "s/$old_scope/$new_scope/g" {} \;
    done
  fi
done

echo "Scope update completed."

