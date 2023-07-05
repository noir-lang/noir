#!/bin/bash
# Executes the bb binary test script.
set -eu

$(aws ecr get-login --region us-east-2 --no-include-email) 2> /dev/null

IMAGE_URI=278380418400.dkr.ecr.us-east-2.amazonaws.com/barretenberg-x86_64-linux-clang-assert:cache-$COMMIT_HASH

docker pull $IMAGE_URI

docker run --rm -t $IMAGE_URI /bin/sh -c "\
  set -e; \
  cd /usr/src/barretenberg/cpp/bin-test; \
  ./bin-test.sh"
