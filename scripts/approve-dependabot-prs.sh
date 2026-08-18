#!/usr/bin/env bash
# Find open dependabot PRs, present them for approval, then approve the
# selected PRs and add them to the merge queue.
#
# Merge-queue note: `gh pr merge` doesn't work on repos with a merge queue,
# so PRs are enqueued via the GraphQL `enqueuePullRequest` mutation instead.
#
# Usage: scripts/approve-dependabot-prs.sh [--dry-run]
#   REPO=owner/name overrides the target repo (default: noir-lang/noir).

set -euo pipefail

REPO="${REPO:-noir-lang/noir}"
DRY_RUN=false
[ "${1:-}" = "--dry-run" ] && DRY_RUN=true

prs=$(gh pr list -R "$REPO" --author "app/dependabot" --state open --limit 100 \
  --json number,title,id,isDraft,reviewDecision,statusCheckRollup)

count=$(jq 'length' <<<"$prs")
if [ "$count" -eq 0 ]; then
  echo "No open dependabot PRs on $REPO."
  exit 0
fi

# Summarize CI as failing > pending > passing from the check rollup.
ci_summary='
  [.statusCheckRollup[]? | .conclusion // .state // "PENDING"] as $s
  | if any($s[]; IN("FAILURE","ERROR","CANCELLED","TIMED_OUT","ACTION_REQUIRED","STARTUP_FAILURE"))
      then "failing"
    elif any($s[]; IN("PENDING","IN_PROGRESS","QUEUED","EXPECTED","WAITING"))
      then "pending"
    else "passing"
    end'

echo "Open dependabot PRs on $REPO:"
echo
jq -r "
  .[] |
  \"  #\(.number)  [ci: \($ci_summary)]\(if .reviewDecision == \"APPROVED\" then \" [approved]\" else \"\" end)\(if .isDraft then \" [draft]\" else \"\" end)  \(.title)\"
" <<<"$prs"
echo

read -rp "Approve + enqueue which PRs? ([a]ll / [p] passing CI only / space-separated PR numbers / [q]uit): " choice
case "$choice" in
  q|Q|"") echo "Aborted."; exit 0 ;;
  a|A|all) selected=$(jq -r '.[].number' <<<"$prs") ;;
  p|P|passing) selected=$(jq -r ".[] | select($ci_summary == \"passing\") | .number" <<<"$prs") ;;
  *) selected=$choice ;;
esac

failures=0
for n in $selected; do
  pr=$(jq --argjson n "$n" '.[] | select(.number == $n)' <<<"$prs")
  if [ -z "$pr" ]; then
    echo "!! #$n is not an open dependabot PR on $REPO, skipping."
    failures=$((failures + 1))
    continue
  fi
  id=$(jq -r '.id' <<<"$pr")
  decision=$(jq -r '.reviewDecision' <<<"$pr")

  if $DRY_RUN; then
    echo "-- dry run: would approve + enqueue #$n"
    continue
  fi

  if [ "$decision" = "APPROVED" ]; then
    echo "-- #$n already approved"
  elif gh pr review -R "$REPO" "$n" --approve; then
    echo "-- #$n approved"
  else
    echo "!! failed to approve #$n, skipping enqueue"
    failures=$((failures + 1))
    continue
  fi

  if output=$(gh api graphql \
      -f query='mutation($id: ID!) {
        enqueuePullRequest(input: { pullRequestId: $id }) {
          mergeQueueEntry { position }
        }
      }' \
      -f id="$id" --jq '.data.enqueuePullRequest.mergeQueueEntry.position' 2>&1); then
    echo "-- #$n added to merge queue (position $output)"
  elif grep -qi "already in the queue" <<<"$output"; then
    echo "-- #$n already in merge queue"
  else
    echo "!! failed to enqueue #$n: $output"
    failures=$((failures + 1))
  fi
done

if [ "$failures" -gt 0 ]; then
  exit 1
fi
