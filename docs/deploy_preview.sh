#!/usr/bin/env bash
set -eu

PR_NUMBER=$1
AZTEC_BOT_COMMENTER_GITHUB_TOKEN="$2"

API_URL="https://api.github.com/repos/AztecProtocol/aztec-packages/pulls/${PR_NUMBER}/files"

echo "API URL: $API_URL"

DOCS_CHANGED=$(curl -L \
    -H "Authorization: Bearer $AZTEC_BOT_COMMENTER_GITHUB_TOKEN" \
    "${API_URL}" | \
    jq '[.[] | select(.filename | startswith("docs/"))] | length > 0')

echo "Docs changed: $DOCS_CHANGED"

if [ "$DOCS_CHANGED" = "false" ]; then
    echo "No docs changed, not deploying"
    exit 0
fi

# Regular deploy if the argument is not "master" and docs changed
DEPLOY_OUTPUT=$(yarn netlify deploy --site aztec-docs-dev)
DOCS_PREVIEW_URL=$(echo "$DEPLOY_OUTPUT" | grep -E "https://.*aztec-docs-dev.netlify.app" | awk '{print $4}')
echo "Unique deploy URL: $DOCS_PREVIEW_URL"

cd ../yarn-project/scripts
AZTEC_BOT_COMMENTER_GITHUB_TOKEN=$AZTEC_BOT_COMMENTER_GITHUB_TOKEN PR_NUMBER=$PR_NUMBER DOCS_PREVIEW_URL=$DOCS_PREVIEW_URL yarn docs-preview-comment 
