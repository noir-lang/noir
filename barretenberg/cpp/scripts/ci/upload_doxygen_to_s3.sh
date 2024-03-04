# Uploads to S3 the contents of a doxygen barretenberg C++ documentation build.
# This is then to be used as a static website.
#!/usr/bin/env bash
[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

retry ecr_login
extract_repo barretenberg-docs /usr/src extracted-repo

BUCKET_NAME="aztec-ci-artifacts"
COMMIT_HASH="${COMMIT_HASH:-$(git rev-parse HEAD)}"
TARGET_FOLDER="barretenberg-doxygen/$COMMIT_HASH"

echo "Uploading to s3://$BUCKET_NAME/$TARGET_FOLDER"
aws s3 cp extracted-repo/src/barretenberg/cpp/docs/build "s3://$BUCKET_NAME/$TARGET_FOLDER" --recursive