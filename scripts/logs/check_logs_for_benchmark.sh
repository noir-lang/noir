#!/usr/bin/env bash
# Checks that all logs needed for assembling aggregate benchmarks have been retrieved.

[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

LOG_FOLDER="${LOG_FOLDER:-log}"
E2E_SRC_FOLDER=/usr/src/yarn-project/end-to-end/src

echo "Checking log files in $LOG_FOLDER"

# Only generate the aggregated benchmark if we've managed to retrieve all the needed log files
# If these runs were skipped due to no changes in their rebuild-patterns, then there's no need
# to recompute the aggregated benchmark. Note that if one benchmark did run but others didn't,
# this skips the whole aggregation. For now, that's fine because all benchmark files have the
# same rebuild pattern rules. But if that changes, then we'd need to go up in the commit history
# to find the latest log files for the unchanged benchmarks.
EXPECTED_LOGS_COUNT=$(find $E2E_SRC_FOLDER -type f -name "bench*.test.ts" | wc -l)
DOWNLOADED_LOGS_COUNT=$(find $LOG_FOLDER -type f -name "*.jsonl" | wc -l)
if [ "$DOWNLOADED_LOGS_COUNT" -lt "$EXPECTED_LOGS_COUNT" ]; then
  echo Found only $DOWNLOADED_LOGS_COUNT out of $EXPECTED_LOGS_COUNT benchmark log files in S3.
  echo Files found: $(find $LOG_FOLDER -type f -name "*.jsonl")
  exit 1
fi

