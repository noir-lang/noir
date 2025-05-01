#!/usr/bin/env bash
set -e

# Run relative to repo root
cd $(dirname "$0")/../

if [[ -z $1 ]]; then
    echo "Must specify repo cache folder" >&2
    exit 1
fi

# if [[ -z $2 ]]; then
#     echo "Must specify git commit" >&2
#     exit 1
# fi
# RUST_COMMIT=$2

CACHE_FOLDER=$1
RUST_REPO=$CACHE_FOLDER/rust

# 'master' on May 1, 2025 @ 19:00 UTC
RUST_COMMIT="0e517d38ad0e72f93c734b14fabd4bb9b7441de6"

# clone if missing from cache
if [ ! -d "$RUST_REPO" ]; then
    echo "$RUST_REPO does not exist: cloning.."
    (cd $CACHE_FOLDER && \
        git clone https://github.com/rust-lang/rust.git)
fi

# checkout target commit
(cd $RUST_REPO && \
    ls && pwd && \
    echo "git fetch" && \
    git fetch && \
    echo "git checkout $RUST_COMMIT" && \
    git checkout $RUST_COMMIT)

echo $CACHE_FOLDER
echo $RUST_REPO

RUST_TEST_FILES=$(find $RUST_REPO/tests -type f -name '*.rs')
NUM_FILES=${#RUST_TEST_FILES}

echo "testing 'nargo compile' against $NUM_FILES inputs.."

for RUST_TEST_FILE in "${RUST_TEST_FILES[@]}"; do
    echo "testing $RUST_TEST_FILE"
    RUST_TEST_RESULT=$(cat $RUST_TEST_FILE | nargo compile --debug-compile-stdin 2>&1)
    if [[ $RUST_TEST_RESULT =~ "The application panicked (crashed)." ]]; then
        echo "Panic found!";
        echo "On commit: $RUST_COMMIT";
        echo "Failing path: $RUST_TEST_FILE";
        echo "Input file:";
        echo "--------------------------------------------------------------------"
        cat $RUST_TEST_FILE
        echo "--------------------------------------------------------------------"
        echo "nargo's result:";
        echo "--------------------------------------------------------------------"
        cat $RUST_TEST_RESULT
        echo "--------------------------------------------------------------------"
        exit 1
    fi
done

echo "$NUM_FILES input files tested."

