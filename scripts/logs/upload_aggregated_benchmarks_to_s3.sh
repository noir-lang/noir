#!/usr/bin/env bash
# Uploads aggregated benchmark logs to S3

[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

BUCKET_NAME="aztec-ci-artifacts"
LOG_FOLDER="${LOG_FOLDER:-log}"
BENCH_FOLDER="${BENCH_FOLDER:-bench}"
COMMIT_HASH="${COMMIT_HASH:-$(git rev-parse HEAD)}"
BENCHMARK_FILE_JSON="${BENCH_FOLDER}/benchmark.json"

# Paths from upload_logs_to_s3
if [ "${BRANCH:-}" = "master" ]; then
  LOG_SOURCE_FOLDER="logs-v1/master/$COMMIT_HASH"
  BARRETENBERG_BENCH_SOURCE_FOLDER="barretenberg-bench-v1/master/$COMMIT_HASH"
  BENCHMARK_TARGET_FILE="benchmarks-v1/master/$COMMIT_HASH.json"
  BENCHMARK_LATEST_FILE="benchmarks-v1/latest.json"
elif [ -n "${PULL_REQUEST:-}" ]; then
  LOG_SOURCE_FOLDER="logs-v1/pulls/${PULL_REQUEST##*/}"
  BARRETENBERG_BENCH_SOURCE_FOLDER="barretenberg-bench-v1/pulls/${PULL_REQUEST##*/}"
  BENCHMARK_TARGET_FILE="benchmarks-v1/pulls/${PULL_REQUEST##*/}.json"
elif [ -n "${CIRCLE_TAG:-}" ]; then
  echo "Skipping benchmark run for ${CIRCLE_TAG} tagged release."
  exit 0
else
  echo "Skipping benchmark run on branch ${BRANCH:-unknown}."
  exit 0
fi

# Upload it to master or pulls
aws s3 cp $BENCHMARK_FILE_JSON "s3://${BUCKET_NAME}/${BENCHMARK_TARGET_FILE}"

# If on master, also update the "latest" benchmark file
if [ -n "${BENCHMARK_LATEST_FILE:-}" ]; then
  aws s3 cp $BENCHMARK_FILE_JSON "s3://${BUCKET_NAME}/${BENCHMARK_LATEST_FILE}"
fi
