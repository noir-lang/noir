#!/usr/bin/env bash
# This script runs the AVM test suite.
# This is essentially a stripped down version of bb-tests.sh.
set -eu

$(aws ecr get-login --region us-east-2 --no-include-email) 2> /dev/null
export PATH="$PATH:$(git rev-parse --show-toplevel)/build-system/scripts"
REPOSITORY=barretenberg-x86_64-linux-clang-assert
# use the image rebuild patterns to compute a content hash, use this to get a URI
IMAGE_URI=$(calculate_image_uri $REPOSITORY)
retry docker pull $IMAGE_URI

docker run --rm -t $IMAGE_URI /bin/sh -c "\
  set -xe; \
  cd /usr/src/barretenberg/cpp; \
  srs_db/download_ignition.sh 1; \
  cd build; \
  ./gtest-parallel/gtest-parallel ./bin/vm_tests --workers=32;"
