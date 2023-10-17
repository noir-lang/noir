#!/bin/bash

extract_repo yarn-project /usr/src project
PROJECT_ROOT=$(pwd)/project/src/

for REPOSITORY in "p2p-bootstrap" "aztec-node" "aztec-faucet"
do
    echo "Deploying $REPOSITORY"
    RELATIVE_PROJECT_DIR=$(query_manifest relativeProjectDir $REPOSITORY)
    cd "$PROJECT_ROOT/$RELATIVE_PROJECT_DIR"

    deploy_ecr $REPOSITORY
done