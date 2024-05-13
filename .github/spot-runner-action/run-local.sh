#!/usr/bin/env bash
set -eu
export GITHUB_REPOSITORY=aztecprotocol/aztec-packages
export INPUT_AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID
export INPUT_AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY
export INPUT_AWS_REGION=us-east-2
export INPUT_EC2_SUBNET_ID=subnet-4cfabd25
export INPUT_EC2_SECURITY_GROUP_ID=sg-0ccd4e5df0dcca0c9
export INPUT_EC2_SPOT_INSTANCE_STRATEGY="BestEffort"
export INPUT_EC2_INSTANCE_TYPE="r6in.32xlarge r6a.32xlarge i4i.32xlarge r7iz.32xlarge"
export INPUT_EC2_AMI_ID=ami-04d8422a9ba4de80f
export INPUT_SUBACTION=start
export INPUT_EC2_INSTANCE_TTL=20
export INPUT_EC2_KEY_NAME="build-instance"
export INPUT_EC2_KEY=$(cat ~/.ssh/build_instance_key | base64)
export INPUT_EC2_INSTANCE_TAGS="[]"
export INPUT_RUNNER_LABEL=""
export GITHUB_ENV=.github-env-mock
export GITHUB_REF=$(git rev-parse HEAD)
cd $(git rev-parse --show-toplevel)
npm -C .github/spot-runner-action run build
node .github/spot-runner-action/dist/index.js