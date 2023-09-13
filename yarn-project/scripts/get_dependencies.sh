#!/bin/bash
set -eu

PROJECT_DIR=$1

echo yarn-project-base
jq -r ".dependencies + .devDependencies | keys | .[] | select(startswith(\"@aztec/\")) | ltrimstr(\"@aztec/\")" $PROJECT_DIR/package.json