#!/usr/bin/env bash
# This script automates the process of benchmarking native on a remote EC2 instance.
# Prerequisites:
# 1. Define the following environment variables:
#    - BB_SSH_KEY: SSH key for EC2 instance, e.g., '-i key.pem'
#    - BB_SSH_INSTANCE: EC2 instance URL
#    - BB_SSH_CPP_PATH: Path to barretenberg/cpp in a cloned repository on the EC2 instance
set -eu

BENCHMARK=${1:-goblin_bench}
COMMAND=${2:-./$BENCHMARK}

# Move above script dir.
cd $(dirname $0)/..

# Create lock file
ssh $BB_SSH_KEY $BB_SSH_INSTANCE "touch ~/BENCHMARKING_IN_PROGRESS"

# Configure and build.
cmake --preset clang16
cmake --build --preset clang16 --target $BENCHMARK

source scripts/_benchmark_remote_lock.sh

cd build
scp $BB_SSH_KEY ./bin/$BENCHMARK $BB_SSH_INSTANCE:$BB_SSH_CPP_PATH/build
ssh $BB_SSH_KEY $BB_SSH_INSTANCE \
  "cd $BB_SSH_CPP_PATH/build ; $COMMAND"
