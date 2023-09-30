#!/bin/bash

# Get current branch name
branch_name=$(git rev-parse --abbrev-ref HEAD)
# Replace `/`` with `-` in branch name
# `/` doesn't work as a version tag in NPM
branch_name=${branch_name//\//-}

# Get latest commit hash (short version)
commit_hash=$(git rev-parse --short HEAD)

# Construct the tag name
tag_name="${branch_name}-${commit_hash}"

# Set version to tag name
newVersion=$tag_name

# Get the locations of all the packages in the yarn workspace
packages=$(yarn workspaces list --json | grep location | awk -F'"' '{print $4}' | grep -v '^.$')

# Check if any packages are found
if [ -z "$packages" ]; then
  echo "No packages found or not a valid Yarn workspace."
  exit 1
fi

# Loop over the packages and update versions
for package_location in $packages; do
  packagePath="$package_location/package.json"

  # Check if package.json exists
  if [ -f "$packagePath" ]; then
    # Extract the current version, get only the SemVer compatible part, and append the newVersion to it
    currentVersion=$(jq -r '.version' "$packagePath")
    
    # Extract the SemVer compatible part using a regex match
    # This is just a safety measure so that the script is idempotent
    # Without it, it will keep appending the newVersion to the version
    semverPart=$(echo "$currentVersion" | grep -oP '^\d+\.\d+\.\d+')
    
    # Append the newVersion to the SemVer part
    appendedVersion="$semverPart-$newVersion"

    # Update the version in package.json
    jq --arg appendedVersion "$appendedVersion" '.version = $appendedVersion' "$packagePath" > "$packagePath.tmp" && mv "$packagePath.tmp" "$packagePath"
    echo "Updated version in $packagePath to $appendedVersion"
  else
    echo "No package.json found in $package_location"
  fi
done