#!/bin/bash
# Executes the bb binary test script.
set -eu

$(aws ecr get-login --region us-east-2 --no-include-email) 2> /dev/null
REPOSITORY=barretenberg-x86_64-linux-clang-assert
IMAGE_URI="278380418400.dkr.ecr.us-east-2.amazonaws.com/$REPOSITORY:cache-$CONTENT_HASH"
docker pull $IMAGE_URI

docker run --rm -t $IMAGE_URI /bin/sh -c "\
  set -xe; \
  cd /usr/src/barretenberg/cpp/bin-test; \
  ./bin-test.sh"
