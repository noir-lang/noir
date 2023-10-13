#!/bin/bash

BRANCH_NAME=$(echo "$BRANCH_NAME" | sed -e "s#refs/[^/]*/##")
DEPLOY_STATUS=$(curl -X GET "https://api.netlify.com/api/v1/sites/1b11824b-a5b7-4872-8c76-aedbe0ac867c/deploys?branch=$BRANCH_NAME"  | jq -r '.[] | select(.created_at != null) | .state' | head -1)

MAX_RETRIES=10
COUNT=0
while [[ "$DEPLOY_STATUS" != "ready" && $COUNT -lt $MAX_RETRIES ]]; do
    sleep 60
    DEPLOY_STATUS=$(curl -X GET "https://api.netlify.com/api/v1/sites/1b11824b-a5b7-4872-8c76-aedbe0ac867c/deploys?branch=$BRANCH_NAME"  | jq -r '.[] | select(.created_at != null) | .state' | head -1)
    COUNT=$((COUNT+1))

    # If deploy status is ready, set the output and exit successfully
    if [[ "$DEPLOY_STATUS" == "ready" ]]; then
        echo "::set-output name=deploy_status::$DEPLOY_STATUS"
        exit 0
    fi
done

echo "Deploy failed or took too long."
echo "::set-output name=deploy_status::failure
exit 1
