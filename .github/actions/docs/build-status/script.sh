#!/usr/bin/env bash

BRANCH_NAME=$(echo "$BRANCH_NAME" | sed -e "s#refs/[^/]*/##")

echo "$SITE_ID"
MAX_RETRIES=10
COUNT=0

# Initial fetch of deploy status
HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "https://api.netlify.com/api/v1/sites/$SITE_ID/deploys?branch=$BRANCH_NAME")
HTTP_CODE=$(echo "$HTTP_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$HTTP_RESPONSE" | sed '$d')

# Check if HTTP request was successful
if [[ "$HTTP_CODE" != "200" ]]; then
    echo "Error: HTTP request failed with status code $HTTP_CODE"
    echo "deploy_status=failure" >> $GITHUB_OUTPUT
    exit 1
fi

# Parse deploy status from response
DEPLOY_STATUS=$(echo "$RESPONSE_BODY" | jq -r '.[] | select(.created_at != null) | .state' | head -1)

# Validate that we got a deploy status
if [[ -z "$DEPLOY_STATUS" ]]; then
    echo "Error: Failed to parse deploy status from API response"
    echo "deploy_status=failure" >> $GITHUB_OUTPUT
    exit 1
fi

while [[ "$DEPLOY_STATUS" != "ready" && $COUNT -lt $MAX_RETRIES ]]; do
    sleep 20
    
    # Fetch deploy status with HTTP status code checking
    HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "https://api.netlify.com/api/v1/sites/$SITE_ID/deploys?branch=$BRANCH_NAME")
    HTTP_CODE=$(echo "$HTTP_RESPONSE" | tail -n1)
    RESPONSE_BODY=$(echo "$HTTP_RESPONSE" | sed '$d')
    
    # Check if HTTP request was successful    
    if [[ "$HTTP_CODE" != "200" ]]; then
        echo "Error: HTTP request failed with status code $HTTP_CODE"
        echo "deploy_status=failure" >> $GITHUB_OUTPUT
        exit 1
    fi
    
    # Parse deploy status from response
    DEPLOY_STATUS=$(echo "$RESPONSE_BODY" | jq -r '.[] | select(.created_at != null) | .state' | head -1)
    
    # Validate that we got a deploy status
    if [[ -z "$DEPLOY_STATUS" ]]; then
        echo "Error: Failed to parse deploy status from API response"
        echo "deploy_status=failure" >> $GITHUB_OUTPUT
        exit 1
    fi
    
    COUNT=$((COUNT+1))
    
    echo "Deploy status: $DEPLOY_STATUS"
    # If deploy status is ready, set the output and exit successfully
    if [[ "$DEPLOY_STATUS" == "ready" ]]; then
        echo "deploy_status=success" >> $GITHUB_OUTPUT
        exit 0
    elif [[ "$DEPLOY_STATUS" == "error" ]]; then
        echo "deploy_status=failure" >> $GITHUB_OUTPUT
        exit 1
    fi
    
    echo "Deploy still running. Retrying..."
done

echo "deploy_status=failure" >> $GITHUB_OUTPUT
exit 1                                            
