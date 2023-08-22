# git subrepo is quite nice, but has one flaw in our workflow:
# We frequently squash commits in PRs, and we might update the .gitrepo file
# with a parent commit that later does not exist. 
# A backup heuristic is used to later find the squashed commit's parent
# using the .gitrepo file's git history. This might be brittle 
# in the face of e.g. a .gitrepo whitespace change, but it's a fallback, 
# we only have this issue in master, and the file should only be edited
# generally by subrepo commands.
set -eu

SUBREPO_PATH="${1:-}"
echo "Auto-fixing squashed parent in $SUBREPO_PATH/.gitrepo."

# Get the commit that last wrote to .gitrepo
last_commit=$(git log -1 --pretty=format:%H -- "$SUBREPO_PATH/.gitrepo")
# Get parent of the last commit
new_parent=$(git log --pretty=%P -n 1 $last_commit)

# Update parent in .gitrepo file
git config --file="$SUBREPO_PATH/.gitrepo" subrepo.parent $new_parent

# Commit this change
git add "$SUBREPO_PATH/.gitrepo"
# This commit should only go into squashed PRs
git commit -m "git_subrepo.sh: Fix parent in .gitrepo file."
