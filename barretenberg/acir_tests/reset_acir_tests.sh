#!/usr/bin/env bash
set -e

# Run from within barretenberg/acir_tests

# Initialize variables for flags
REBUILD_NARGO_FLAG=""
PROGRAMS=""

# Parse the arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --rebuild-nargo)
            REBUILD_NARGO_FLAG="--rebuild-nargo"
            ;;
        --programs)
            shift
            PROGRAMS="$@"
            break  # Exit loop after collecting all programs
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
    shift
done

# Clean and rebuild noir, then compile the test programs if --rebuild-nargo flag is set
cd ../../noir/noir-repo

if [[ -n "$REBUILD_NARGO_FLAG" ]]; then
    cargo clean
    noirup -p .
fi

# Rebuild test programs with rebuild.sh
cd test_programs
if [[ -n "$PROGRAMS" ]]; then
    ./rebuild.sh $PROGRAMS
else
    ./rebuild.sh
fi

# Remove and repopulate the test artifacts in bberg
cd ../../../barretenberg/acir_tests
rm -rf acir_tests
./clone_test_vectors.sh
