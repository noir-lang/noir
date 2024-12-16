#!/usr/bin/env bash
set -e

BACKEND=${BACKEND:-bb}

ARTIFACT_PATH=$1
ARTIFACT_NAME=$(basename "$ARTIFACT_PATH")

GATES_INFO=$($BACKEND gates -b $ARTIFACT_PATH)
MAIN_FUNCTION_INFO=$(echo $GATES_INFO | jq -r '.functions[0] | .name = "main"')
echo "{\"programs\": [ {\"package_name\": \"$ARTIFACT_NAME\", \"functions\": [$MAIN_FUNCTION_INFO]} ]}"

