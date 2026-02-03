#!/bin/bash
set -eu

BACKEND=${BACKEND:-bb}

nargo compile --force

# Just the command to make sure we don't get any error from Barretenberg due to CLI arg mismatch.
noir-profiler gates --artifact-path ./target/example.json --backend-path $BACKEND --output target
