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

CRITICAL_LIBRARIES=$(yq '.libraries | filter(.critical == true) | map("https://github.com/" + .repo) | .[]' ./EXTERNAL_NOIR_LIBRARIES.yml)
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

# Check if yarn is installed, install if not
checkYarnInstalled() {
    if ! command -v yarn &> /dev/null; then
        echo "yarn not found, installing..." >&2
        if command -v npm &> /dev/null; then
            npm install -g yarn
        elif command -v corepack &> /dev/null; then
            corepack enable
        else
            echo "Error: yarn is required but not available. Please install yarn or npm." >&2
            exit 1
        fi
    fi
}

# Run tests for a library directory
runLibraryTests() {
    LIB_DIR=$1
    
    (
        cd $LIB_DIR
        
        # Check if library has package.json (needs yarn setup)
        if [ -f "package.json" ]; then
            echo "Detected package.json, setting up yarn dependencies..."
            checkYarnInstalled
            yarn install --frozen-lockfile 2>&1 || yarn install 2>&1
        fi
        
        # Check for custom test script
        if [ -f "scripts/run.sh" ]; then
            echo "Running custom test script: scripts/run.sh"
            chmod +x scripts/run.sh
            ./scripts/run.sh
        elif [ -f "package.json" ] && grep -q '"test"' package.json; then
            echo "Running yarn test"
            yarn test
        else
            # Default: run nargo test
            echo "Running nargo test"
            nargo test -q
        fi
    )
}

for REPO in ${REPOS_TO_CHECK[@]}; do
    echo "Checking $REPO"
    TMP_DIR=$(mktemp -d)
    
    TAG=$(getLatestReleaseTagForRepo $REPO)
    git clone $REPO -c advice.detachedHead=false --depth 1 --branch $TAG $TMP_DIR
    
    runLibraryTests $TMP_DIR

    rm -rf $TMP_DIR
done
