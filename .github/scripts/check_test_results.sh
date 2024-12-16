#!/bin/bash
set -eu

# Usage: ./check_test_results.sh <expected.json> <actual.jsonl>
# Compares the output of two test results of the same repository.
# If any of the files doesn't exist or is empty, the script will consider that the test suite
# couldn't be compiled.

if [ -s $1 ] && [ -s $2 ]; then
  # Both files exist, let's compare them
  cat $1 | jq -c 'select(.type == "test" and .event != "started") | {suite: .suite, name: .name, status: .event}' | jq -s -c 'sort_by(.suite, .name) | .[]' > $1.jq
  cat $2 | jq -c 'select(.type == "test" and .event != "started") | {suite: .suite, name: .name, status: .event}' | jq -s -c 'sort_by(.suite, .name) | .[]' > $2.jq

  diff $1.jq $2.jq
elif [ -s $1] ; then
  # Only the expected file exists, which means the actual test couldn't be compiled.
  echo "Error: external library tests couldn't be compiled. You could rename $1 to $1.does_not_compile if it's expected that the external library can't be compiled."
  exit -1
elif [ -s $2]; then
  # Only the actual file exists, which means we are expecting the external library
  # not to compile but it did.
  echo "Error: expected external library not to compile, but it did. Please run its tests with 'nargo test --format json' and save that output to $1"
  exit -1
else
  # Both files don't exists, which means we are expecting the external library not
  # to compile, and it didn't, so all is good.
  exit 0
fi