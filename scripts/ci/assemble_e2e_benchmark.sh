#!/usr/bin/env bash
# Grabs the log files uploaded in build-system/scripts/upload_logs_to_s3
# that contain representative benchmarks, extracts whatever metrics are interesting,
# and assembles a single file that shows the current state of the repository.

[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

BUCKET_NAME="aztec-ci-artifacts"
LOG_FOLDER="${LOG_FOLDER:-log}"
BENCH_FOLDER="${BENCH_FOLDER:-bench}"
COMMIT_HASH="${COMMIT_HASH:-$(git rev-parse HEAD)}"
BASE_BENCH_PATH=""
BENCHMARK_FILE_JSON="${BENCH_FOLDER}/benchmark.json"
BASE_BENCHMARK_FILE_JSON="${BENCH_FOLDER}/base-benchmark.json"

# Paths from build-system/scripts/upload_logs_to_s3
if [ "${CIRCLE_BRANCH:-}" = "master" ]; then
  LOG_SOURCE_FOLDER="logs-v1/master/$COMMIT_HASH"
  BARRETENBERG_BENCH_SOURCE_FOLDER="barretenberg-bench-v1/master/$COMMIT_HASH"
  BENCHMARK_TARGET_FILE="benchmarks-v1/master/$COMMIT_HASH.json"
  BENCHMARK_LATEST_FILE="benchmarks-v1/latest.json"
elif [ -n "${CIRCLE_PULL_REQUEST:-}" ]; then
  LOG_SOURCE_FOLDER="logs-v1/pulls/${CIRCLE_PULL_REQUEST##*/}"
  BARRETENBERG_BENCH_SOURCE_FOLDER="barretenberg-bench-v1/pulls/${CIRCLE_PULL_REQUEST##*/}"
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
EXPECTED_LOGS_COUNT=$(find yarn-project/end-to-end/src -type f -name "bench*.test.ts" | wc -l)
DOWNLOADED_LOGS_COUNT=$(find $LOG_FOLDER -type f -name "*.jsonl" | wc -l)
if [ "$DOWNLOADED_LOGS_COUNT" -lt "$EXPECTED_LOGS_COUNT" ]; then
  echo Found $DOWNLOADED_LOGS_COUNT out of $EXPECTED_LOGS_COUNT benchmark log files in s3://${BUCKET_NAME}/${LOG_SOURCE_FOLDER}/. Exiting.
  exit 1
fi

# Download barretenberg log files, these are direct benchmarks and separate from the above
aws s3 cp "s3://${BUCKET_NAME}/${BARRETENBERG_BENCH_SOURCE_FOLDER}/" $LOG_FOLDER --exclude '*' --include '*_bench.json' --recursive

# Generate the aggregated benchmark file
mkdir -p $BENCH_FOLDER
CONTAINER_BENCH_FOLDER="/usr/src/yarn-project/bench"
CONTAINER_LOG_FOLDER="/usr/src/yarn-project/log"
export DOCKER_RUN_OPTS="\
 -v $(realpath $BENCH_FOLDER):${CONTAINER_BENCH_FOLDER}:rw \
 -e BENCH_FOLDER=${CONTAINER_BENCH_FOLDER} \
 -v $(realpath $LOG_FOLDER):${CONTAINER_LOG_FOLDER}:rw \
 -e LOG_FOLDER=${CONTAINER_LOG_FOLDER} \
 -e BASE_BENCH_PATH \
 -e AZTEC_BOT_COMMENTER_GITHUB_TOKEN \
 -e CIRCLE_PULL_REQUEST"
yarn-project/scripts/run_script.sh workspace @aztec/scripts bench-aggregate
echo "generated: $BENCHMARK_FILE_JSON"

# Upload it to master or pulls
aws s3 cp $BENCHMARK_FILE_JSON "s3://${BUCKET_NAME}/${BENCHMARK_TARGET_FILE}"

# If on master, also update the "latest" benchmark file
if [ -n "${BENCHMARK_LATEST_FILE:-}" ]; then
  aws s3 cp $BENCHMARK_FILE_JSON "s3://${BUCKET_NAME}/${BENCHMARK_LATEST_FILE}"
fi

# If on a pull request, get the data from the most recent commit on master where it's available,
# generate a markdown comment, and post it on the pull request
if [ -n "${CIRCLE_PULL_REQUEST:-}" ]; then
  MASTER_COMMIT_HASH=$(curl -s "https://api.github.com/repos/AztecProtocol/aztec-packages/pulls/${CIRCLE_PULL_REQUEST##*/}" | jq -r '.base.sha')
  MASTER_COMMIT_HASHES=($(git log $MASTER_COMMIT_HASH --format="%H" -n 50))

  set +e
  echo "Searching for base benchmark data starting from commit $MASTER_COMMIT_HASH"
  for commit_hash in "${MASTER_COMMIT_HASHES[@]}"; do
    aws s3 cp "s3://${BUCKET_NAME}/benchmarks-v1/master/$commit_hash.json" $BASE_BENCHMARK_FILE_JSON
    if [ $? -eq 0 ]; then
      echo "Downloaded base data from commit $commit_hash"
      export BASE_BENCH_PATH=master/$commit_hash
      break;
    fi
  done
  set -e

  if [ -z "${BASE_BENCH_PATH:-}" ]; then
    echo "No base commit data found"
  fi

  (yarn-project/scripts/run_script.sh workspace @aztec/scripts bench-comment && echo "commented on pr $CIRCLE_PULL_REQUEST") || echo "failed commenting on pr"
fi
