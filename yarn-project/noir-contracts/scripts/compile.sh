#!/bin/bash

# Compiles noir contracts in parallel, bubbling any compilation errors

ROOT=$(pwd)

# Error flag file
error_file="/tmp/error.$$"
# Array of child PIDs
pids=()

# Handler for SIGCHLD, cleanup if child exit with error
handle_sigchld() {
    for pid in "${pids[@]}"; do
        # If process is no longer running
        if ! kill -0 "$pid" 2>/dev/null; then
            # Wait for the process and get exit status
            wait "$pid"
            status=$?

            # If exit status is error
            if [ $status -ne 0 ]; then
                # Create error file
                touch "$error_file"
            fi
        fi
    done
}

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

# Check nargo version matches the expected one
echo "Using $(nargo --version)"
EXPECTED_VERSION=$(jq -r '.commit' ../noir-compiler/src/noir-version.json)
FOUND_VERSION=$(nargo --version | grep -oP 'git version hash: \K[0-9a-f]+')
if [ "$EXPECTED_VERSION" != "$FOUND_VERSION" ]; then
  echo "Expected nargo version $EXPECTED_VERSION but found version $FOUND_VERSION. Aborting."
  exit 1
fi


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
