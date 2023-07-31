#!/bin/bash
set -eu

# Usage: ./this.sh <branch> <commit_message>
# Script for migrating PRs from barretenberg repo to aztec-packages.
# Meant to be used from master with a branch name that exists on barretenberg repo but not aztec-packages.
# You can change the commit message after with git commit --amend if needed.

# Display usage if not enough arguments
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <branch> <commit_message>"
    exit 1
fi

# Check for unstaged changes
if ! git diff-index --quiet HEAD --; then
    echo "Error: You have unstaged changes. Please commit or stash them before running git_subrepo.sh."
    exit 1
fi

# Check if the current branch is master
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" != "master" ]; then
    echo "Error: This script must be run from the 'master' branch. Current branch is '$current_branch'"
    exit 1
fi

# Fetch the latest updates from origin
git fetch origin

# Check if master is same as origin/master
if ! git diff --quiet master origin/master; then
    echo "Error: Local 'master' branch is not up to date with 'origin/master'. Please pull the latest changes."
    exit 1
fi

BRANCH="$1"
COMMIT_MESSAGE="$2"
SUBREPO_PATH=circuits/cpp/barretenberg # can be changed to another subrepo if useful

SCRIPT_DIR=$(dirname "$(realpath "$0")")
cd "$SCRIPT_DIR"/..

echo "(branch migrate) Switching to a new branch named '$BRANCH' (this branch can't already exist)"

# Check if branch already exists
if git rev-parse --verify "$BRANCH" >/dev/null 2>&1; then
    echo "Error: Aztec branch '$BRANCH' already exists. Please delete it with 'git branch -D $BRANCH' if you are sure you don't need it."
    exit 1
fi

git checkout -b "$BRANCH"

echo "(branch migrate) Pulling from upstream barretenberg repo. If this doesn't work, your barretenberg branch may need to merge barretenberg master."
if ! scripts/git_subrepo.sh pull "$SUBREPO_PATH" --branch=$BRANCH; then
    echo "Error: Failed to pull from upstream barretenberg repo. Check your branch name or network connection."
    exit 1
fi

echo "(branch migrate) Automatic git data fix"
# Tosses away the .gitrepo changes, as those we only want if pulling from barretenberg master, not PRs (which will go in as aztec commits).
# because git-subrepo uses 'git rm -r', we fix up .gitmodules after as well. This is an edge-case gotcha using 
# git submodules along with git-subrepo.

git checkout HEAD^ -- "$SUBREPO_PATH"/.gitrepo .gitmodules

if ! git commit --amend -m "$COMMIT_MESSAGE"; then
    echo "Error: Failed to commit changes. Check your commit message."
    exit 1
fi
echo "(branch migrate) All done. You can now 'git push origin HEAD' and click to create a PR on aztec. Changes will then automatically go into barretenberg when merged."
