#!/bin/bash
set -eo pipefail

# Helper function for building packages in yarn project
build_package() {
  local package_name="$1"
  local build_command="${2:-yarn build}"

  echo "Building $package_name..."
  (cd "yarn-project/$package_name" && $build_command)
}

# Build script. If run on Netlify, first it needs to compile all yarn-projects 
# that are involved in typedoc in order to generate their type information.
if [ -n "$NETLIFY" ]; then
  # Move to project root
  cd ..
  echo Working dir $(pwd)

  # Make sure the latest tag is available for loading code snippets from it
  LAST_TAG="aztec-packages-v$(jq -r '.["."]' .release-please-manifest.json)"
  echo Fetching latest released tag $LAST_TAG...
  git fetch origin refs/tags/$LAST_TAG:refs/tags/$LAST_TAG

  # Tweak global tsconfig so we skip tests in all projects
  echo Removing test files from tsconfig...
  jq '. + { "exclude": ["**/*test.ts"] }' yarn-project/tsconfig.json > yarn-project/tsconfig.tmp.json
  mv yarn-project/tsconfig.tmp.json yarn-project/tsconfig.json

  # Install deps (maybe we can have netlify download these automatically so they get cached..?)
  echo Installing yarn-project dependencies...
  (cd yarn-project && yarn)

  # Build the required projects for typedoc
  build_package "aztec-rpc"
  build_package "aztec.js" "yarn build:ts"

  # Back to docs site
  cd docs

  # Install deps
  echo Install docs deps...
  yarn
fi

# Now build the docsite
echo Building docsite...
yarn preprocess && yarn typedoc && yarn docusaurus build
