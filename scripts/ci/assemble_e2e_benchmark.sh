#!/usr/bin/env bash
# Grabs the log files uploaded in build-system/scripts/upload_logs_to_s3
# that contain representative benchmarks, extracts whatever metrics are interesting,
# and assembles a single file that shows the current state of the repository.

[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace

# Enter yarn-project dir.
cd "$(dirname $0)/../../yarn-project/"
set -eu

PULL_REQUEST_ID=$1
BRANCH=$2
BUCKET_NAME="aztec-ci-artifacts"
COMMIT_HASH=$3
BASE_BENCH_PATH=""

# Paths from scripts/upload_logs_to_s3
if [ "${BRANCH:-}" = "master" ]; then
  LOG_SOURCE_FOLDER="logs-v1/master/$COMMIT_HASH"
  BARRETENBERG_BENCH_SOURCE_FOLDER="barretenberg-bench-v1/master/$COMMIT_HASH"
  BENCHMARK_TARGET_FILE="benchmarks-v1/master/$COMMIT_HASH.json"
  BENCHMARK_LATEST_FILE="benchmarks-v1/latest.json"
elif [ -n "$PULL_REQUEST_ID" ]; then
  LOG_SOURCE_FOLDER="logs-v1/pulls/${PULL_REQUEST_ID##*/}"
  BARRETENBERG_BENCH_SOURCE_FOLDER="barretenberg-bench-v1/pulls/${PULL_REQUEST_ID##*/}"
  BENCHMARK_TARGET_FILE="benchmarks-v1/pulls/${PULL_REQUEST_ID##*/}.json"
elif [ -n "${CIRCLE_TAG:-}" ]; then
  echo "Skipping benchmark run for ${CIRCLE_TAG} tagged release."
  exit 0
else
  echo "Skipping benchmark run on branch ${BRANCH:-unknown}."
  exit 0
fi

# Download benchmark log files from S3 LOG_SOURCE_FOLDER into local 'log' folder
mkdir -p log
aws s3 cp "s3://${BUCKET_NAME}/${LOG_SOURCE_FOLDER}/" log --exclude '*' --include 'bench*.jsonl' --recursive

# Only generate the aggregated benchmark if we've managed to retrieve all the needed log files
# If these runs were skipped due to no changes in their rebuild-patterns, then there's no need
# to recompute the aggregated benchmark. Note that if one benchmark did run but others didn't,
# this skips the whole aggregation. For now, that's fine because all benchmark files have the
# same rebuild pattern rules. But if that changes, then we'd need to go up in the commit history
# to find the latest log files for the unchanged benchmarks.
EXPECTED_LOGS_COUNT=$(find end-to-end/src -type f -name "bench*.test.ts" | wc -l)
DOWNLOADED_LOGS_COUNT=$(find log -type f -name "*.jsonl" | wc -l)
if [ "$DOWNLOADED_LOGS_COUNT" -lt "$EXPECTED_LOGS_COUNT" ]; then
  echo Found $DOWNLOADED_LOGS_COUNT out of $EXPECTED_LOGS_COUNT benchmark log files in s3://${BUCKET_NAME}/${LOG_SOURCE_FOLDER}/. Exiting.
  exit 0
fi

# Download barretenberg log files, these are direct benchmarks and separate from the above
aws s3 cp "s3://${BUCKET_NAME}/${BARRETENBERG_BENCH_SOURCE_FOLDER}/" log --exclude '*' --include '*_bench.json' --recursive

# Generate the aggregated benchmark file
# TODO rename CIRCLE_PULL_REQUEST to PULL_REQUEST_ID
CIRCLE_PULL_REQUEST=$PULL_REQUEST_ID LOG_FOLDER=$(pwd)/log yarn workspace @aztec/scripts bench-aggregate
echo "generated: scripts/bench/benchmark.json"

# Upload it to master or pulls
aws s3 cp scripts/bench/benchmark.json "s3://${BUCKET_NAME}/${BENCHMARK_TARGET_FILE}"

# If on master, also update the "latest" benchmark file
if [ -n "${BENCHMARK_LATEST_FILE:-}" ]; then
  aws s3 cp scripts/bench/benchmark.json "s3://${BUCKET_NAME}/${BENCHMARK_LATEST_FILE}"
fi
