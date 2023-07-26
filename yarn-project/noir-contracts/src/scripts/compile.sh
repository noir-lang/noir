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

# Build contracts
for CONTRACT_NAME in "$@"; do
  CONTRACT_FOLDER="${CONTRACT_NAME}_contract"
  echo "Compiling $CONTRACT_NAME..."
  cd src/contracts/$CONTRACT_FOLDER
  rm -f target/*

  # If VERBOSE is not set, compile with 'nargo' and redirect standard error (stderr) to /dev/null and standard output (stdout) to /dev/null.
  # If the compilation fails, rerun the compilation with 'nargo' and show the compiler output.
  if [[ -z "${VERBOSE:-}" ]]; then
    "$NARGO_COMMAND" compile main --experimental-ssa --contracts 2> /dev/null > /dev/null  || (echo "Error compiling contract. Re-running as verbose to show compiler output:"; "$NARGO_COMMAND" compile main --experimental-ssa --contracts);
  else
    "$NARGO_COMMAND" compile main --experimental-ssa --contracts
  fi

  cd $ROOT
  echo "Copying output for $CONTRACT_NAME"
  NODE_OPTIONS=--no-warnings yarn ts-node --esm src/scripts/copy_output.ts $CONTRACT_NAME
  
  echo "Formatting contract folders"
  yarn run -T prettier -w ./src/artifacts/$CONTRACT_FOLDER.json ../aztec.js/src/abis/*.json ./src/types/*.ts
  echo -e "Done\n"

done

# Check for stale artifacts
for json_path in src/artifacts/*.json; do
  json_file="$(basename "$json_path")";
  contract_name="${json_file%.json}";
  if [ ! -d "./src/contracts/$contract_name" ]; then
    echo "WARN: Source code for artifact '$contract_name' not found. Consider deleting the artifact.";
  fi
done
