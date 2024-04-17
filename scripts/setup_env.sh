#!/bin/bash

# Setup environment variables
echo "Setting up environment variables..."
echo FORCE_COLOR=1 >> $GITHUB_ENV

# Docker login
echo "Logging in to Docker..."
echo $1 | docker login -u aztecprotocolci --password-stdin

# Make earthly-ci script available
echo "PATH=$(dirname $(realpath $0)):$PATH" >> $GITHUB_ENV
echo "EARTHLY_CONFIG=$(git rev-parse --show-toplevel)/.github/earthly-ci-config.yml" >> $GITHUB_ENV