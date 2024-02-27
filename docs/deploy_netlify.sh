#!/usr/bin/env bash
[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

extract_repo docs /usr/src extracted-repo
cd extracted-repo/src/docs
npm install netlify-cli -g

DEPLOY_OUTPUT=""

# Check if we're on master
if [ "$1" = "master" ]; then
    # Deploy to production if the argument is "master"
    DEPLOY_OUTPUT=$(netlify deploy --site aztec-docs-dev --prod)
else    
    # TODO we should prob see if check_rebuild can be used for this
    PR_URL="$2"
    API_URL="${PR_URL/github.com/api.github.com/repos}"
    API_URL="${API_URL/pull/pulls}"
    API_URL="${API_URL}/files"

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
    DEPLOY_OUTPUT=$(netlify deploy --site aztec-docs-dev)
    UNIQUE_DEPLOY_URL=$(echo "$DEPLOY_OUTPUT" | grep -E "https://.*aztec-docs-dev.netlify.app" | awk '{print $4}')
    echo "Unique deploy URL: $UNIQUE_DEPLOY_URL"

    extract_repo yarn-project /usr/src project
    cd project/src/yarn-project/scripts

    UNIQUE_DEPLOY_URL=$UNIQUE_DEPLOY_URL yarn docs-preview-comment
fi
