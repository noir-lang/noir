#!/bin/bash

BRANCH_NAME=$(echo "$BRANCH_NAME" | sed -e "s#refs/[^/]*/##")
DEPLOY_STATUS=$(curl -X GET "https://api.netlify.com/api/v1/sites/1b11824b-a5b7-4872-8c76-aedbe0ac867c/deploys?branch=$BRANCH_NAME"  | jq -r '.[] | select(.created_at != null) | .state' | head -1)

MAX_RETRIES=10
COUNT=0
while [[ "$DEPLOY_STATUS" != "ready" && $COUNT -lt $MAX_RETRIES ]]; do
sleep 60
DEPLOY_STATUS=$(curl -X GET "https://api.netlify.com/api/v1/sites/1b11824b-a5b7-4872-8c76-aedbe0ac867c/deploys?branch=$BRANCH_NAME"  | jq -r '.[] | select(.created_at != null) | .state' | head -1)
COUNT=$((COUNT+1))

echo $DEPLOY_STATUS
done

# If deploy status isn't ready after all retries, fail the CI
if [[ "$DEPLOY_STATUS" != "ready" ]]; then
echo "Deploy failed or took too long."
exit 1
fi
