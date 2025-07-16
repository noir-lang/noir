#!/bin/bash

set -eux

# Configuration
ORIGINAL_PR_NUMBER=$1
REPO='noir-lang/noir'
NEW_BRANCH="chore/typo-redo-$ORIGINAL_PR_NUMBER"
AUTHOR=$(gh pr view $ORIGINAL_PR_NUMBER --json author --jq '.author.login')

# Step 1: Checkout the PR locally
echo "Checking out PR #$ORIGINAL_PR_NUMBER"
gh pr checkout $ORIGINAL_PR_NUMBER --branch "typo-pr-branch"

# Step 2: Create new branch from master for squash commit
echo "Creating new local branch $NEW_BRANCH from master"
git checkout -b $NEW_BRANCH master

# Step 3: Squash merge the PR branch onto the new branch
echo "Squashing PR branch onto $NEW_BRANCH"
git merge "typo-pr-branch" --squash

# Step 4: Commit squash changes
echo "Committing squash commit"
git commit -m "chore: redo typo PR"

# Step 5: Push the new branch to GitHub
echo "Pushing new branch $NEW_BRANCH to GitHub"
git push origin $NEW_BRANCH

# Step 6: Create a new pull request
echo "Creating a new pull request for $NEW_BRANCH"
gh pr create --base master --head $NEW_BRANCH --title "chore: redo typo PR by $AUTHOR" --body "Thanks $AUTHOR for https://github.com/$REPO/pull/$ORIGINAL_PR_NUMBER. Our policy is to redo typo changes to dissuade metric farming. This is an automated script."

# Step 7: Close the original PR
echo "Closing original PR #$ORIGINAL_PR_NUMBER"
gh pr close $ORIGINAL_PR_NUMBER

# Step 8: Delete the temporary branch
git branch -D typo-pr-branch

echo "Script completed."
