#!/bin/bash

DIST_TAG=${1:-"latest"}

extract_repo yarn-project /usr/src project
PROJECT_ROOT=$(pwd)/project/src/

for REPOSITORY in "aztec-sandbox" "cli"; do
  echo "Deploying $REPOSITORY $DIST_TAG"
  RELATIVE_PROJECT_DIR=$(query_manifest relativeProjectDir $REPOSITORY)
  cd "$PROJECT_ROOT/$RELATIVE_PROJECT_DIR"

  deploy_dockerhub $REPOSITORY x86_64 $DIST_TAG
  deploy_dockerhub $REPOSITORY arm64 $DIST_TAG
  create_dockerhub_manifest $REPOSITORY x86_64,arm64 $DIST_TAG
done
