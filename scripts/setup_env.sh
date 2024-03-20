#!/bin/bash

# Setup environment variables
echo "Setting up environment variables..."
echo FORCE_COLOR=1 >> $GITHUB_ENV
echo DOCKER_HOST=ssh://build-instance-$1.aztecprotocol.com >> $GITHUB_ENV

# Docker login
echo "Logging in to Docker..."
echo $2 | docker login -u aztecprotocolci --password-stdin

# Configure SSH
echo "Configuring SSH..."
mkdir -p ~/.ssh
echo $3 | base64 -d > ~/.ssh/build_instance_key
chmod 600 ~/.ssh/build_instance_key
cat > ~/.ssh/config <<EOF
IdentityFile ~/.ssh/build_instance_key
StrictHostKeyChecking no
User ubuntu
EOF

# Install earthly
$(dirname $0)/earthly --version
echo "PATH=$(dirname $(realpath $0)):$PATH" >> $GITHUB_ENV