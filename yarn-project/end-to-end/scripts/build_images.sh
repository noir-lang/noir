#!/bin/bash
set -e

SCRIPT_DIR=$(dirname "$(realpath "$0")")

CREDENTIALS_FILE="$HOME/.aws/credentials"
AWS_ACCESS_KEY_ID=$(grep -oP '(?<=aws_access_key_id=).*' "$CREDENTIALS_FILE")
AWS_SECRET_ACCESS_KEY=$(grep -oP '(?<=aws_secret_access_key=).*' "$CREDENTIALS_FILE")

TARGETS=(
    "$SCRIPT_DIR/../+export-aztec"
    "$SCRIPT_DIR/+export-spartan-test"
)

for TARGET in "${TARGETS[@]}"; do
    # make temp file for build logs within the script dir
    LOG_FILE="$SCRIPT_DIR/build_$(basename "$TARGET").log"
    rm -f "$LOG_FILE"
    touch "$LOG_FILE"

    echo "Building image for $TARGET"
    echo "Logging to $LOG_FILE"

    earthly \
    --secret AWS_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID" \
    --secret AWS_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY" \
    "$TARGET" > "$LOG_FILE" 2>&1

    if [ $? -ne 0 ]; then
        echo "Build failed for $TARGET. Check $LOG_FILE for more information"
        continue
    fi

    # the \\ is becuase the target starts with a + and we need to escape it
    IMAGE_NAME=$(grep -oP "Image .*\\$(basename "$TARGET") output as \K.*" "$LOG_FILE")

    if [ -z "$IMAGE_NAME" ]; then
        echo "Failed to extract image name for $TARGET. Check $LOG_FILE for more information"
        continue
    fi

    echo "Built image $IMAGE_NAME"

    kind load docker-image "$IMAGE_NAME"

    echo "Loaded image into kind cluster"
done