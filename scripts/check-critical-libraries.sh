#!/usr/bin/env bash
set -e

# Run relative to repo root
cd $(dirname "$0")/../

if [[ -z $1 ]]; then
    echo "Must specify Noir release to test against" >&2
    echo "usage: ./check-critical-libraries.sh <release-version>" >&2
    exit 1
fi
noirup -v $1

CRITICAL_LIBRARIES=$(grep -v "^#\|^$" ./CRITICAL_NOIR_LIBRARIES)
readarray -t REPOS_TO_CHECK < <(echo "$CRITICAL_LIBRARIES")

getLatestReleaseTagForRepo() {
    REPO_NAME=$1
    TAG=$(gh release list -R $REPO_NAME --json 'tagName,isLatest' -q '.[] | select(.isLatest == true).tagName')
    if [[ -z $TAG ]]; then
        echo "$REPO_NAME has no valid release" >&2
        exit 1
    fi
    echo $TAG
}

for REPO in ${REPOS_TO_CHECK[@]}; do
    echo $REPO   
    TMP_DIR=$(mktemp -d)
    
    TAG=$(getLatestReleaseTagForRepo $REPO)
    git clone $REPO -c advice.detachedHead=false --depth 1 --branch $TAG $TMP_DIR
    
    nargo test -q --program-dir $TMP_DIR

    rm -rf $TMP_DIR
done
