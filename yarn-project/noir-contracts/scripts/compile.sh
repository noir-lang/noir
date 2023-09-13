#!/bin/bash

# Compiles Aztec.nr contracts in parallel, bubbling any compilation errors

source ./scripts/catch.sh
source ./scripts/nargo_check.sh

ROOT=$(pwd)

# Error flag file
error_file="/tmp/error.$$"
# Array of child PIDs
pids=()

# Set SIGCHLD handler
trap handle_sigchld SIGCHLD # Trap any ERR signal and call the custom error handler

build() {
  CONTRACT_NAME=$1
  CONTRACT_FOLDER="${CONTRACT_NAME}_contract"
  echo "Compiling $CONTRACT_NAME..."
  rm -f target/${CONTRACT_FOLDER}-*
  rm -f target/debug_${CONTRACT_FOLDER}-*

  # If the compilation fails, rerun the compilation with 'nargo' and show the compiler output.
  nargo compile --package $CONTRACT_FOLDER --output-debug;
}

# Check nargo version
nargo_check

# Build contracts
for CONTRACT_NAME in "$@"; do
  build $CONTRACT_NAME &
  pids+=($!)
done

# Wait for all background processes to finish
wait

# If error file exists, exit with error
if [ -f "$error_file" ]; then
    rm "$error_file"
    echo "Error occurred in one or more child processes. Exiting..."
    exit 1
fi
