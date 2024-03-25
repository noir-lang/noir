#!/bin/bash

# Setup environment variables
echo "Setting up environment variables..."
echo FORCE_COLOR=1 >> $GITHUB_ENV

# Docker login
echo "Logging in to Docker..."
echo $1 | docker login -u aztecprotocolci --password-stdin

# Make earthly-cloud and earthly-cloud-bench scripts available
echo "PATH=$(dirname $(realpath $0)):$PATH" >> $GITHUB_ENV
echo "GITHUB_ACTOR=$2" >> $GITHUB_ENV