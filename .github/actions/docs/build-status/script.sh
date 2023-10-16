#!/bin/bash

BRANCH_NAME=$(echo "$BRANCH_NAME" | sed -e "s#refs/[^/]*/##")
DEPLOY_STATUS=$(curl -X GET "https://api.netlify.com/api/v1/sites/$SITE_ID/deploys?branch=$BRANCH_NAME"  | jq -r '.[] | select(.created_at != null) | .state' | head -1)

echo "$SITE_ID"
MAX_RETRIES=10
COUNT=0
while [[ "$DEPLOY_STATUS" != "ready" && $COUNT -lt $MAX_RETRIES ]]; do
    sleep 20
    DEPLOY_STATUS=$(curl -X GET "https://api.netlify.com/api/v1/sites/$SITE_ID/deploys?branch=$BRANCH_NAME"  | jq -r '.[] | select(.created_at != null) | .state' | head -1)
    COUNT=$((COUNT+1))

    echo "Deploy status: $DEPLOY_STATUS"
    # If deploy status is ready, set the output and exit successfully
    if [[ "$DEPLOY_STATUS" == "ready" ]]; then
        echo "deploy_status=success" >> $GITHUB_OUTPUT
        exit 0
    elif [[ "$DEPLOY_STATUS" == "error" ]]; then
        echo "deploy_status=failure" >> $GITHUB_OUTPUT
        exit 0
    fi

    echo "Deploy still running. Retrying..."
done

echo "deploy_status=failure" >> $GITHUB_OUTPUT
exit 0
