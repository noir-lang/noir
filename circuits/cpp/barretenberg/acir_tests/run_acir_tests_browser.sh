#!/bin/bash
set -em

cleanup() {
  lsof -i ":8080" | awk 'NR>1 {print $2}' | xargs kill -9
  exit
}

trap cleanup SIGINT SIGTERM

# Skipping firefox because this headless firefox is so slow.
export BROWSER=${BROWSER:-chrome,webkit}

# Can be "mt" or "st".
THREAD_MODEL=${THREAD_MODEL:-mt}

# TODO: Currently webkit doesn't seem to have shared memory so is a single threaded test regardless of THREAD_MODEL!
echo "Testing thread model: $THREAD_MODEL"
(cd browser-test-app && yarn serve:dest:$THREAD_MODEL) > /dev/null 2>&1 &
sleep 1
VERBOSE=1 BIN=./headless-test/bb.js.browser ./run_acir_tests.sh $@
lsof -i ":8080" | awk 'NR>1 {print $2}' | xargs kill -9