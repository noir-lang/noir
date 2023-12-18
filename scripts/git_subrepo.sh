#!/usr/bin/env bash
set -eu

SCRIPT_DIR=$(dirname "$(realpath "$0")")

# Check for unstaged changes
if ! git diff-index --quiet HEAD --; then
    echo "Error: You have unstaged changes. Please commit or stash them before running git_subrepo.sh."
    exit 1
fi

# git subrepo is quite nice, but has one flaw in our workflow:
# We frequently squash commits in PRs, and we might update the .gitrepo file
# with a parent commit that later does not exist.
# A backup heuristic is used to later find the squashed commit's parent
# using the .gitrepo file's git history. This might be brittle
# in the face of e.g. a .gitrepo whitespace change, but it's a fallback,
# we only have this issue in master, and the file should only be edited
# generally by subrepo commands.
SUBREPO_PATH="${2:-}"
if [ -d "$SUBREPO_PATH" ] ; then
    # Read parent commit from .gitrepo file
    parent_commit=$(awk -F'= ' '/parent =/{print $2}' $SUBREPO_PATH/.gitrepo)
    # Check if the parent commit exists in this branch
    if ! git branch --contains $parent_commit | grep -q '\*'; then
	"$SCRIPT_DIR"/fix_subrepo_edge_case.sh "$SUBREPO_PATH"
    fi
fi

# Function to handle subrepo actions and possible error
run_subrepo() {
    "$SCRIPT_DIR/git-subrepo/lib/git-subrepo" "$@" 2>&1 | tee /tmp/subrepo_output.log
    local exit_code="${PIPESTATUS[0]}"  # Capture the exit status of git-subrepo command

    if [ $exit_code -ne 0 ]; then
        # Check for the specific error message and maybe recover
        if grep -q "doesn't contain upstream HEAD" /tmp/subrepo_output.log; then
            "$SCRIPT_DIR/fix_subrepo_edge_case.sh" "$SUBREPO_PATH"
            echo "Rerunning..."
            "$SCRIPT_DIR/git-subrepo/lib/git-subrepo" "$@"
        else
            exit $exit_code
        fi
    fi
}


run_subrepo "$@"
