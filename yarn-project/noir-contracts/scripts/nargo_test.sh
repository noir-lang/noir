#!/bin/bash

# Tests noir contracts, if multiple are provided, then they are testing in parallel, bubbling any testing errors
#
# Usage:
# If testing a single contract:
#   ./scripts/test.sh CONTRACT <CONTRACT_NAME>
# If testing multiple contracts:
#   ./scripts/test.sh CONTRACT <CONTRACT_NAME> <CONTRACT_NAME> <CONTRACT_NAME> <CONTRACT_NAME> ...
# If testing a library:
#  ./scripts/test.sh LIB <LIBRARY_NAME>
# If testing multiple libraries:
#  ./scripts/test.sh LIB <LIBRARY_NAME> <LIBRARY_NAME> <LIBRARY_NAME> <LIBRARY_NAME> ...

ROOT=$(pwd)

# Get the project type from the first argument
PROJECT_TYPE=$1
shift

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

test() {
  PROJECT_NAME=$1

  if [ "$PROJECT_TYPE" == "CONTRACT" ]; then
    CONTRACT_FOLDER="${PROJECT_NAME}_contract"
    echo "Testing contract $PROJECT_NAME..."
    cd src/contracts/$CONTRACT_FOLDER
    nargo test
  else
    echo "Testing library $PROJECT_NAME..."
    cd ../noir-libs/$PROJECT_NAME
    nargo test
  fi
}

echo "Using $(nargo --version)"

# Build contracts
for PROJECT_NAME in "$@"; do
  test $PROJECT_NAME  &
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
