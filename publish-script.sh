#!/bin/bash
set -e

# Get current branch name
branch_name=$(git rev-parse --abbrev-ref HEAD)

# Get latest commit hash (short version)
commit_hash=$(git rev-parse --short HEAD)

# Construct the tag name
tag_name="${branch_name}-${commit_hash}"

# Check if working directory is clean
# if [ -n "$(git status --porcelain)" ]; then
#   echo "Working directory is not clean, please commit or stash your changes before publishing"
#   exit 1
# fi

# Check if yarn is installed
if ! command -v yarn &> /dev/null; then
  echo "Yarn is not installed, please install it first"
  exit 1
fi

# Get the list of workspace packages
packages=$(yarn workspaces list --json | grep location | awk -F'"' '{print $4}' | grep -v '^.$')

# Record the package name and version
declare -A package_versions

# Loop over the packages and pack them, also adjust the package.json for local dependencies
for package_location in $packages; do
  cd $package_location
  
  # Extract the package name and version from the package.json
  package_name=$(cat package.json | jq -r .name)
  package_version=$(cat package.json | jq -r .version)
  
  package_versions["$package_name"]=$package_version
  
  cd -
done

# Adjust Dependency Versions and Publish
for package_location in $packages; do
  cd $package_location
  
  # Extract the package name from the package.json
  package_name=$(cat package.json | jq -r .name)

  # Check if the package is private
  is_private=$(cat package.json | jq -r .private)
  if [ "$is_private" == "true" ]; then
    cd -
    continue
  fi

  # Adjust the dependency versions in package.json
  for dep_name in "${!package_versions[@]}"; do
    dep_version=${package_versions["$dep_name"]}
    jq '.dependencies //= {} | .devDependencies //= {} | .dependencies |= with_entries(if .key == "'$dep_name'" then .value="'$dep_version'" else . end) | .devDependencies |= with_entries(if .key == "'$dep_name'" then .value="'$dep_version'" else . end)' package.json > package.json.tmp
    mv package.json.tmp package.json
  done
  
  # Publish the package with the constructed tag name
  npm publish --tag dev --access public
  
  cd -
done