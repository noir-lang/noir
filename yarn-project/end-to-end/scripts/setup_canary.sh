#!/bin/bash
set -eu

COMMIT_TAG=$1
TARGET_PKGS_FILE=$2

# Check if file exists and read it into an array
if [ -f "$TARGET_PKGS_FILE" ]; then
  mapfile -t TARGET_PKGS < <(cat "$TARGET_PKGS_FILE")
  echo "Loaded array:"
  for i in "${TARGET_PKGS[@]}"; do
    echo "$i"
  done
else
  echo "File $TARGET_PKGS_FILE does not exist."
fi

if [ -z "$COMMIT_TAG" ]; then
  echo "No commit tag provided."
  exit 0
fi

VERSION=$(npx semver $COMMIT_TAG)
if [ -z "$VERSION" ]; then
  echo "$COMMIT_TAG is not a semantic version."
  exit 1
fi

echo "Removing all files & folders that aren't needed for canary tests"
TARGET_DIR="./src"
cd "$TARGET_DIR"

# Loop through all files and folders in the directory
for item in $(ls -A); do
  if [[ "$item" != "index.ts" && "$item" != "canary" ]]; then
    # Remove the item (either file or folder)
    rm -rf "$item"
  fi
done
cd ..

echo "Updating external Aztec dependencies to version $VERSION"

# Packages that are publically available in npm
# TARGET_PKGS=("@aztec/aztec.js" "@aztec/cli" "@aztec/l1-artifacts" "@aztec/noir-contracts")

TMP=$(mktemp)
for PKG in "${TARGET_PKGS[@]}"; do
  jq --arg v $VERSION ".dependencies[\"$PKG\"] = \$v" package.json > $TMP && mv $TMP package.json
done

jq ".references = []" tsconfig.json > $TMP && mv $TMP tsconfig.json
