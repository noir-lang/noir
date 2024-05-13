#!/bin/bash

COMMIT_HASH=$(git rev-parse HEAD)

# Setup environment variables
echo "Setting up environment variables..."
echo "FORCE_COLOR=1" >> $GITHUB_ENV
echo "EARTHLY_BUILD_ARGS=PULL_REQUEST=$PULL_REQUEST,BRANCH=$BRANCH,COMMIT_HASH=$COMMIT_HASH" >> $GITHUB_ENV

# Docker login
echo "Logging in to Docker..."
echo $1 | docker login -u aztecprotocolci --password-stdin

# Make earthly-ci script available
echo "PATH=$(dirname $(realpath $0)):$PATH" >> $GITHUB_ENV
echo "EARTHLY_CONFIG=$(git rev-parse --show-toplevel)/.github/earthly-ci-config.yml" >> $GITHUB_ENV
echo ECR_REGION=us-east-2 >> $GITHUB_ENV
echo AWS_ACCOUNT=278380418400 >> $GITHUB_ENV
echo ECR_URL=278380418400.dkr.ecr.us-east-2.amazonaws.com >> $GITHUB_ENV
echo ECR_DEPLOY_REGION=eu-west-2 >> $GITHUB_ENV