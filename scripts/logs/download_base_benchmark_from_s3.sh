#!/usr/bin/env bash
# Downloads base benchmarks from S3 to compare with the current benchmarks via bench-comment

[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

BUCKET_NAME="aztec-ci-artifacts"
BENCH_FOLDER="${BENCH_FOLDER:-bench}"
COMMIT_HASH="${COMMIT_HASH:-$(git rev-parse HEAD)}"
BASE_BENCHMARK_FILE_JSON="${BENCH_FOLDER}/base-benchmark.json"

# If on a pull request, get the data from the most recent commit on master where it's available to generate a comment comparing them
if [ -n "${PULL_REQUEST:-}" ]; then
  MASTER_COMMIT_HASH=$(curl -s "https://api.github.com/repos/AztecProtocol/aztec-packages/pulls/${PULL_REQUEST##*/}" | jq -r '.base.sha')
  MASTER_COMMIT_HASHES=($(git log $MASTER_COMMIT_HASH --format="%H" -n 50))

  mkdir -p $BENCH_FOLDER

  set +e
  echo "Searching for base benchmark data starting from commit $MASTER_COMMIT_HASH"
  for commit_hash in "${MASTER_COMMIT_HASHES[@]}"; do
    aws s3 cp "s3://${BUCKET_NAME}/benchmarks-v1/master/$commit_hash.json" $BASE_BENCHMARK_FILE_JSON
    if [ $? -eq 0 ]; then
      echo "Downloaded base data from commit $commit_hash"
      exit 0
    fi
  done
  set -e

  echo "No base commit data found"
else
  echo "Not on a pull request, skipping download of base benchmark data"
fi
