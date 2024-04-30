#!/usr/bin/env bash
# Downloads the log files uploaded in upload_logs_to_s3

set -eu

BUCKET_NAME="aztec-ci-artifacts"
LOG_FOLDER="${LOG_FOLDER:-log}"
COMMIT_HASH="${COMMIT_HASH:-$(git rev-parse HEAD)}"

echo "Downloading logs from S3 for commit $COMMIT_HASH in branch ${BRANCH:-} at pull request ${PULL_REQUEST:-none}"

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
else
  echo "Skipping benchmark run on branch ${BRANCH:-unknown}."
  exit 0
fi

mkdir -p $LOG_FOLDER

# Download benchmark log files from S3 LOG_SOURCE_FOLDER into local LOG_FOLDER
echo "Downloading benchmark log files from $BUCKET_NAME/$LOG_SOURCE_FOLDER to $LOG_FOLDER"
aws s3 cp "s3://${BUCKET_NAME}/${LOG_SOURCE_FOLDER}/" $LOG_FOLDER --exclude '*' --include 'bench*.jsonl' --recursive

# Download barretenberg log files, these are direct benchmarks and separate from the above
aws s3 cp "s3://${BUCKET_NAME}/${BARRETENBERG_BENCH_SOURCE_FOLDER}/" $LOG_FOLDER --exclude '*' --include '*_bench.json' --recursive

echo "Downloaded log files $(ls $LOG_FOLDER)"