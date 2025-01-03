#!/bin/bash
set -eu

# Usage: ./check_test_results.sh <expected.json> <actual.jsonl>
# Compares the output of two test results of the same repository.
# If any of the files doesn't exist or is empty, the script will consider that the test suite
# couldn't be compiled.

function process_json_lines() {
  cat $1 | jq -c 'select(.type == "test" and .event == "failed") | {suite: .suite, name: .name}' | jq -s -c 'sort_by(.suite, .name) | .[]' > $1.jq
}

if [ -f $1 ] && [ -f $2 ]; then
  # Both files exist, let's compare them
  $(process_json_lines $2)
  if ! diff $1 $2.jq; then
    echo "Error: test failures don't match expected failures"
    echo "Lines prefixed with '>' are new test failures (you could add them to '$1')"
    echo "Lines prefixed with '<' are tests that were expected to fail but passed (you could remove them from '$1')"
  fi
elif [ -f $1 ]; then
  # Only the expected file exists, which means the actual test couldn't be compiled.
  echo "Error: external library tests couldn't be compiled."
  echo "You could rename '$1' to '$1.does_not_compile' if it's expected that the external library can't be compiled."
  exit -1
elif [ -f $2 ]; then
  # Only the actual file exists, which means we are expecting the external library
  # not to compile but it did.
  echo "Error: expected external library not to compile, but it did."
  echo "You could create '$1' with these contents:"
  $(process_json_lines $2)
  cat $2.jq
  exit -1
else
  # Both files don't exists, which means we are expecting the external library not
  # to compile, and it didn't, so all is good.
  exit 0
fi