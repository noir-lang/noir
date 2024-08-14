#!/bin/bash

set -eux

# Configuration
ORIGINAL_PR_NUMBER=$1
REPO='noir-lang/noir'
NEW_BRANCH="chore/typo-redo-$ORIGINAL_PR_NUMBER"
AUTHOR=`gh pr view $ORIGINAL_PR_NUMBER --json author --jq '.author.login'`

# Step 1: Checkout the PR locally
echo "Checking out PR #$ORIGINAL_PR_NUMBER"
gh pr checkout $ORIGINAL_PR_NUMBER

# Step 2: Create a new local branch
echo "Creating new local branch $NEW_BRANCH"
git checkout -b $NEW_BRANCH

# Step 3: Squash commits
echo "Squashing new local branch $NEW_BRANCH"
git reset --soft master
git add .
git commit -m "chore: typo fixes"

# Step 4: Push the new branch to GitHub
echo "Pushing new branch $NEW_BRANCH to GitHub"
git push origin $NEW_BRANCH

# Step 5: create a new pull request
echo "Creating a new pull request for $NEW_BRANCH"
gh pr create --base master --head $NEW_BRANCH --title "chore: redo typo PR by $AUTHOR" --body "Thanks $AUTHOR for https://github.com/$REPO/pull/$ORIGINAL_PR_NUMBER. Our policy is to redo typo changes to dissuade metric farming. This is an automated script."

# Step 6: Close the original PR
echo "Closing original PR #$ORIGINAL_PR_NUMBER"
gh pr close $ORIGINAL_PR_NUMBER

echo "Script completed."
