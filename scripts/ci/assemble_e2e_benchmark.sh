#!/bin/bash
# Grabs the log files uploaded in yarn-project/end-to-end/scripts/upload_logs_to_s3.sh
# that contain representative benchmarks, extracts whatever metrics are interesting,
# and assembles a single file that shows the current state of the repository.

[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

BUCKET_NAME="aztec-ci-artifacts"
LOG_FOLDER="${LOG_FOLDER:-log}"
COMMIT_HASH="${COMMIT_HASH:-$(git rev-parse HEAD)}"
BENCHMARK_FILE_JSON="benchmark.json"

# Adapted from yarn-project/end-to-end/scripts/upload_logs_to_s3.sh
if [ "${CIRCLE_BRANCH:-}" = "master" ]; then
  LOG_SOURCE_FOLDER="logs-v1/master/$COMMIT_HASH"
  BENCHMARK_TARGET_FILE="benchmarks-v1/master/$COMMIT_HASH.json"
  BENCHMARK_LATEST_FILE="benchmarks-v1/latest.json"
elif [ -n "${CIRCLE_PULL_REQUEST:-}" ]; then
  LOG_SOURCE_FOLDER="logs-v1/pulls/${CIRCLE_PULL_REQUEST##*/}"
  BENCHMARK_TARGET_FILE="benchmarks-v1/pulls/${CIRCLE_PULL_REQUEST##*/}.json"
elif [ -n "${CIRCLE_TAG:-}" ]; then
  echo "Skipping benchmark run for ${CIRCLE_TAG} tagged release."
  exit 0
else
  echo "Skipping benchmark run on branch ${CIRCLE_BRANCH:-unknown}."
  exit 0
fi

# Download benchmark log files from S3 LOG_SOURCE_FOLDER into local LOG_FOLDER
mkdir -p $LOG_FOLDER
aws s3 cp "s3://${BUCKET_NAME}/${LOG_SOURCE_FOLDER}/" $LOG_FOLDER --exclude '*' --include 'bench*.jsonl' --recursive

# Only generate the aggregated benchmark if we've managed to retrieve all the needed log files
# If these runs were skipped due to no changes in their rebuild-patterns, then there's no need
# to recompute the aggregated benchmark. Note that if one benchmark did run but others didn't,
# this skips the whole aggregation. For now, that's fine because all benchmark files have the
# same rebuild pattern rules. But if that changes, then we'd need to go up in the commit history
# to find the latest log files for the unchanged benchmarks.
EXPECTED_BENCHMARK_COUNT=$(find yarn-project/end-to-end/src -type f -name "bench*.test.ts" | wc -l)
DOWNLOADED_BENCHMARK_COUNT=$(find $LOG_FOLDER -type f -name "*.jsonl" | wc -l)
if [ "$DOWNLOADED_BENCHMARK_COUNT" -lt "$EXPECTED_BENCHMARK_COUNT" ]; then
  echo Found $DOWNLOADED_BENCHMARK_COUNT out of $EXPECTED_BENCHMARK_COUNT benchmark log files in s3://${BUCKET_NAME}/${LOG_SOURCE_FOLDER}/. Exiting.
  exit 0
fi

# Generate the aggregated benchmark file
node scripts/ci/aggregate_e2e_benchmark.js
echo "generated: $BENCHMARK_FILE_JSON"

# Upload it to master or pulls
aws s3 cp $BENCHMARK_FILE_JSON "s3://${BUCKET_NAME}/${BENCHMARK_TARGET_FILE}"

# If on master, also update the "latest" benchmark file
if [ -n "${BENCHMARK_LATEST_FILE:-}" ]; then
  aws s3 cp $BENCHMARK_FILE_JSON "s3://${BUCKET_NAME}/${BENCHMARK_LATEST_FILE}"
fi

# If on a pull request, comment on it
if [ -n "${CIRCLE_PULL_REQUEST:-}" ]; then
  (node scripts/ci/comment_e2e_benchmark.js && echo "commented on pr $CIRCLE_PULL_REQUEST") || echo "failed commenting on pr"
fi


