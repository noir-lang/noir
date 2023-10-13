#!/bin/bash

ACTION_PATH=$1

BRANCH_NAME=$(echo "$BRANCH_NAME" | sed -e "s#refs/[^/]*/##")

COMMENT_TEMPLATE=$(cat "$ACTION_PATH/comment-template.md")
COMMENT_BODY="${COMMENT_TEMPLATE//BRANCH_NAME_PLACEHOLDER/$BRANCH_NAME}"

COMMENTS_URL="https://api.github.com/repos/noir-lang/noir/issues/$PR_NUMBER/comments"
EXISTING_COMMENT_ID=$(curl -sSL -H "Authorization: token $GITHUB_TOKEN" $COMMENTS_URL | jq '.[] | select(.user.login == "github-actions[bot]") | select (.body == "'"$COMMENT_BODY"'") | .id')

echo "Comments url: $COMMENTS_URL"
echo "Existing comment ID: $EXISTING_COMMENT_ID"

COMMENT_UPDATE_URL="https://api.github.com/repos/noir-lang/noir/issues/comments"
echo "Comment update URL: $COMMENT_UPDATE_URL"

# If comment exists, update it; otherwise, create a new one
if [[ -n "$EXISTING_COMMENT_ID" ]]; then
echo "Updating comment..."

echo "Comment body: $COMMENT_BODY"
echo "Comment update URL: $COMMENT_UPDATE_URL/$EXISTING_COMMENT_ID"

curl -sSL \
    -H "Authorization: token $GITHUB_TOKEN" \
    -H "Content-Type: application/json" \
    -X PATCH \
    -d "{\"body\": \"$COMMENT_BODY\"}" \
    "$COMMENT_UPDATE_URL/$EXISTING_COMMENT_ID"

echo "Comment updated."
else
echo "Creating comment..."

echo "Comment body: $COMMENT_BODY"
echo "Comment URL: $COMMENTS_URL"

curl -sSL \
    -H "Authorization: token $GITHUB_TOKEN" \
    -H "Content-Type: application/json" \
    -X POST \
    -d "{\"body\": \"$COMMENT_BODY\"}" \
    $COMMENTS_URL

echo "Comment created."
fi
