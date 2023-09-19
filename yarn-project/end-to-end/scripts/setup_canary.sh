#!/bin/bash
set -eu

COMMIT_TAG=$1
TARGET_PKGS_FILE=$2

# Check if file exists and read it into an array
if [ -f "$TARGET_PKGS_FILE" ]; then
  mapfile -t TARGET_PKGS < <(cat "$TARGET_PKGS_FILE")
  echo "Loaded package array:"
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

set +e  # Temporarily disable exit on error
VERSION=$(npx semver $COMMIT_TAG)
RESULT=$?  # Capture the exit status of the last command
set -e  # Re-enable exit on error

if [ $RESULT -ne 0 ]; then
  echo "Error when running 'npx semver' with commit tag: $COMMIT_TAG"
  exit 1
fi

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
JSON_TARGET_PKGS=$(printf '%s\n' "${TARGET_PKGS[@]}" | jq -R -s -c 'split("\n") | map(select(. != ""))')

TMP=$(mktemp)
jq --arg v $VERSION --argjson target_pkgs "$JSON_TARGET_PKGS" '
.dependencies |= with_entries(
  select(
    (.key | startswith("@aztec")) as $isAztec |
    if $isAztec then
      .key as $k | any($target_pkgs[]; . == $k)
    else
      true
    end
  ) |
  if .key as $k | any($target_pkgs[]; . == $k) then
    .value = $v
  else
    .
  end
)' package.json > $TMP && mv $TMP package.json

jq ".references = []" tsconfig.json > $TMP && mv $TMP tsconfig.json
