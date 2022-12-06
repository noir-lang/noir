#!/bin/bash
# This script builds the monorepo projects listed in the build_local script, terminating when it reaches TARGET_PROJECT.
# It performs the build inside a docker image that can launch further docker builds by mapping the users docker daemon
# socket into the container. A kind of simulated "docker-in-docker".
# We copy the monorepo entire working tree into this container, excluding any build output.
# The mechanics of this are, we mount the users repo into the container, then clone it into a directory within the
# container, and also apply modified/untracked/deleted changes to the internal repo.
# The result is we have a fresh copy of the repo, with only the working changes applied, that we can modify as we wish.

set -e

TARGET_PROJECT=$1
REPO=aztec2-internal
COMMIT_HASH=$(git rev-parse HEAD)

# If we're calling this script from within a project directory, that's the target project.
if [ -z "$TARGET_PROJECT" ]; then
  TARGET_PROJECT=$(git rev-parse --show-prefix)
  if [ -n "$TARGET_PROJECT" ]; then
    # We are in a project folder.
    ONLY_TARGET=true
    TARGET_PROJECT=$(basename $TARGET_PROJECT)
    cd $(git rev-parse --show-cdup)
  fi
fi

docker build -t $REPO-build - <<EOF
FROM ubuntu:latest
RUN apt update && apt install -y git rsync docker.io
EOF

docker run -ti --rm -v/run/user/$UID/docker.sock:/var/run/docker.sock -v$(git rev-parse --show-toplevel):/repo:ro $REPO-build /bin/bash -c "
# Checkout head.
mkdir /$REPO
cd /$REPO
git init
git remote add origin /repo
git fetch --depth 1 origin $COMMIT_HASH
git checkout FETCH_HEAD

# Copy untracked and modified files, and remove deleted files, from our current repo.
cd /repo
{ git ls-files --others --exclude-standard ; git diff --name-only --diff-filter=TMAR HEAD ; } | rsync -a --files-from=- . /$REPO
for F in \$(git ls-files --deleted); do rm /$REPO/\$F > /dev/null 2>&1; done

# Setup build environment.
cd /$REPO/.circleci
source ./setup_env $COMMIT_HASH '' mainframe_$USER /repo
cd ..

build_local $TARGET_PROJECT $ONLY_TARGET
"