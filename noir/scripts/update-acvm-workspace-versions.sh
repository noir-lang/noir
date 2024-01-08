#!/bin/bash

# Path to the top-level Cargo.toml
TOP_LEVEL_CARGO_TOML="Cargo.toml"

# Temporary file to store paths
TEMP_PATHS_FILE="temp_paths.txt"

# Extract local dependencies paths that start with "acvm-repo"
grep 'path = "acvm-repo' "$TOP_LEVEL_CARGO_TOML" | sed -E 's/.*path = "([^"]+)".*/\1/' > "$TEMP_PATHS_FILE"

# Read each path and update the version in the top level workspace Cargo.toml file
# to match the version in the component's Cargo.toml
while IFS= read -r component_path; do
    # Extract the component name
    # This is acir_field for example or stdlib
    component_name=$(basename "$component_path")

    # Extract the version from the component's Cargo.toml
    # This is acvm-repo/acir_field for example
    component_version=$(grep '^version =' "$component_path/Cargo.toml" | sed -E 's/version = "([^"]+)"/\1/')

    if [ -z "$component_version" ]; then
        echo "Error: Unable to extract version for $component_name from $component_path/Cargo.toml"
        continue
    fi

    # Update the version in the top-level Cargo.toml
    sed -i.bak -E "s|($component_name[[:space:]]*=[[:space:]]*\{[[:space:]]*version[[:space:]]*=[[:space:]]*\")([^\"]+)(\".*)|\1$component_version\3|" "$TOP_LEVEL_CARGO_TOML"

    if [ $? -ne 0 ]; then
        echo "Error: Unable to update version for $component_name in $TOP_LEVEL_CARGO_TOML"
    else
        echo "Version for $component_name updated successfully to $component_version in $TOP_LEVEL_CARGO_TOML"
    fi
done < "$TEMP_PATHS_FILE"

# Remove temporary file and backup file
rm "$TEMP_PATHS_FILE"
rm "${TOP_LEVEL_CARGO_TOML}.bak"
