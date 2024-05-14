#!/usr/bin/env bash

# Uploads to S3 the contents of the log file mounted on the end-to-end container,
# which contains log entries with an associated event and metrics for it.
# Logs are uploaded to aztec-ci-artifacts/logs-v1/master/$COMMIT/$JOB.jsonl
# or to aztec-ci-artifacts/logs-v1/pulls/$PRNUMBER/$JOB.jsonl if on a PR

[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

LOG_FOLDER=$1
BUCKET_NAME="aztec-ci-artifacts"
COMMIT_HASH="${COMMIT_HASH:-$(git rev-parse HEAD)}"

echo "Uploading logs to S3 for commit $COMMIT_HASH in branch ${BRANCH:-} at pull request ${PULL_REQUEST:-none}"

if [ ! -d "$LOG_FOLDER" ] || [ -z "$(ls -A "$LOG_FOLDER")" ]; then
  echo "No logs in folder $LOG_FOLDER to upload"
  exit 0
fi

# Paths used in scripts/ci/assemble_e2e_benchmark.sh
if [ "${BRANCH:-}" = "master" ]; then
  TARGET_FOLDER="logs-v1/master/$COMMIT_HASH/"
elif [ -n "${PULL_REQUEST:-}" ]; then
  TARGET_FOLDER="logs-v1/pulls/${PULL_REQUEST##*/}"
fi

if [ -n "${TARGET_FOLDER:-}" ]; then
  aws s3 cp $LOG_FOLDER "s3://${BUCKET_NAME}/${TARGET_FOLDER}"  --include "*.jsonl" --recursive
else
  echo Skipping upload since no target folder was defined
fi