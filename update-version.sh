#!/bin/bash
set -e

# Get current branch name
branch_name=$(git rev-parse --abbrev-ref HEAD)
# Replace `/`` with `-` in branch name
# `/` doesn't work as a version tag in NPM
branch_name=${branch_name//\//-}

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

# Loop over the packages and update versions, also adjust the package.json for local dependencies
for package_location in $packages; do
  packagePath="$package_location/package.json"
  
  if [ -f "$packagePath" ]; then
    # Extract the package name and current version from the package.json
    package_name=$(jq -r '.name' "$packagePath")
    currentVersion=$(jq -r '.version' "$packagePath")
    
    # Extract the SemVer compatible part using a regex match
    semverPart=$(echo "$currentVersion" | grep -oP '^\d+\.\d+\.\d+')
    
    # Append the newVersion to the SemVer part
    appendedVersion="$semverPart-$tag_name"
    
    # Update the version in package.json
    jq --arg appendedVersion "$appendedVersion" '.version = $appendedVersion' "$packagePath" > "$packagePath.tmp" && mv "$packagePath.tmp" "$packagePath"
    echo "Updated version in $packagePath to $appendedVersion"
    
    # Record the updated version
    package_versions["$package_name"]=$appendedVersion
  else
    echo "No package.json found in $package_location"
  fi
done

# Adjust Dependency Versions
for package_location in $packages; do
  packagePath="$package_location/package.json"
  
  if [ -f "$packagePath" ]; then
    # Check if the package is private
    is_private=$(jq -r '.private' "$packagePath")
    if [ "$is_private" == "true" ]; then
      continue
    fi
    
    # Adjust the dependency versions in package.json
    for dep_name in "${!package_versions[@]}"; do
      dep_version=${package_versions["$dep_name"]}
      jq '.dependencies //= {} | .devDependencies //= {} | .dependencies |= with_entries(if .key == "'$dep_name'" then .value="'$dep_version'" else . end) | .devDependencies |= with_entries(if .key == "'$dep_name'" then .value="'$dep_version'" else . end)' "$packagePath" > "$packagePath.tmp" && mv "$packagePath.tmp" "$packagePath"
    done
  else
    echo "No package.json found in $package_location"
  fi
done
