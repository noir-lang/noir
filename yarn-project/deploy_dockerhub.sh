#!/bin/bash

extract_repo yarn-project /usr/src project
PROJECT_ROOT=$(pwd)/project/src/

for REPOSITORY in "pxe" "aztec-sandbox"
do
    echo "Deploying $REPOSITORY"
    RELATIVE_PROJECT_DIR=$(query_manifest relativeProjectDir $REPOSITORY)
    cd "$PROJECT_ROOT/$RELATIVE_PROJECT_DIR"

    deploy_dockerhub $REPOSITORY x86_64
    deploy_dockerhub $REPOSITORY arm64
    create_dockerhub_manifest $REPOSITORY x86_64,arm64
done
