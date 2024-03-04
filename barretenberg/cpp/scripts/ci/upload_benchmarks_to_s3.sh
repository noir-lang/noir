# Uploads to S3 a recent barretenberg benchmark run.
#!/usr/bin/env bash
[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

retry ecr_login
extract_repo barretenberg-bench /usr/src extracted-repo

BUCKET_NAME="aztec-ci-artifacts"
COMMIT_HASH="${COMMIT_HASH:-$(git rev-parse HEAD)}"

if [ "${BRANCH:-}" = "master" ]; then
  TARGET_FOLDER="barretenberg-bench-v1/master/$COMMIT_HASH/"
elif [ -n "${PULL_REQUEST:-}" ]; then
  TARGET_FOLDER="barretenberg-bench-v1/pulls/${PULL_REQUEST##*/}"
else
  echo Skipping upload since no target folder was defined
  exit
fi
echo "Uploading to s3://$BUCKET_NAME/$TARGET_FOLDER"
aws s3 cp extracted-repo/src/barretenberg/cpp/build/ultra_honk_rounds_bench.json "s3://$BUCKET_NAME/$TARGET_FOLDER/ultra_honk_rounds_bench.json"
