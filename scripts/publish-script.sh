#!/bin/bash
set -e

# Run script from root of repo 
cd ..

# Check if yarn is installed
if ! command -v yarn &> /dev/null; then
  echo "Yarn is not installed, please install it first"
  exit 1
fi

# Get the list of workspace packages
packages=$(yarn workspaces list --json | grep location | awk -F'"' '{print $4}' | grep -v '^.$')

# Loop over the packages and publish them
for package_location in $packages; do
  cd $package_location
  
  # Extract the package name from the package.json
  package_name=$(jq -r .name package.json)

  # Check if the package is private
  is_private=$(jq -r .private package.json)
  if [ "$is_private" == "true" ]; then
    cd -
    continue
  fi
  
  # Publish the package with the constructed tag name
  npm publish --tag dev --access public
  
  cd -
done