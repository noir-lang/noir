#!/bin/bash

# Example:

# If you've compiled Noir from source:
# ./compile.sh --nargo-path=path/to/nargo --verbose zk_token ecdsa_account
# yarn noir:build --nargo-path=path/to/nargo zk_token ecdsa_account

# If nargo is installed properly in your PATH:
# yarn noir:build zk_token ecdsa_account

# Enable strict mode:
# Exit on error (set -e), treat unset variables as an error (set -u),
# and propagate the exit status of the first failing command in a pipeline (set -o pipefail).
set -euo pipefail;

ROOT=$(pwd)
NARGO_COMMAND="nargo"  # Default nargo command

# Function to display script usage
usage() {
  echo "Usage: $0 [--nargo-path=<path>] [--verbose] CONTRACT_NAME [CONTRACT_NAME...]"
  echo "Arguments:"
  echo "  --nargo-path=<path>  Specify the path to the 'nargo' executable (optional)."
  echo "  --verbose            Enable verbose compilation output (optional)."
  echo "  CONTRACT_NAME        Name of the contract(s) to compile and process (omitting the '_contract' suffix)."
  exit 1
}

# Parse command-line arguments
for arg in "$@"; do
  case $arg in
    --nargo-path=*) # Optional.
      NARGO_COMMAND="${arg#*=}"  # Extract the value after '--nargo-path='
      NARGO_COMMAND=$(eval echo "$NARGO_COMMAND")  # Expand tilde (~) in the path to be the home directory (for example)
      shift  # Move to the next command-line argument
      ;;
    --verbose) # Optional.
      # Set the VERBOSE environment variable to enable verbose mode
      export VERBOSE=1
      shift  # Move to the next command-line argument
      ;;
    *)
      # If an unrecognized argument is provided, we assume it is a CONTRACT_NAME
      # and break out of the loop to start processing the contracts.
      break
      ;;
  esac
done

# Check if at least one CONTRACT_NAME is provided, if not, display usage information.
if [ $# -eq 0 ]; then
  usage
  exit 0
fi

echo "Using $($NARGO_COMMAND --version)"

# Run build scripts
./scripts/compile.sh "$@"
./scripts/types.sh "$@"