#!/usr/bin/env bash
# This script takes the state of your current repository, and clones it inside of a docker container.
# You likely don't have a fresh clone, and it's paramount that to test bootstrapping, we don't have any
# intermediate build state in the context.
# To achieve this we mount your working directory into the container, and then perform the clone into the container.
# After cloning the repo we build the relevant dockerfile to bootstrap.
# "docker-in-docker" is achieved by mounting the host systems docker socket into the container.

DOCKERFILE=${1:-Dockerfile.lunar}

docker build -t bootstrap-build - <<EOF
FROM ubuntu:latest
RUN apt update && apt install -y git rsync docker.io
EOF

docker run -ti --rm -v/run/user/$UID/docker.sock:/var/run/docker.sock -v$(git rev-parse --show-toplevel):/repo:ro bootstrap-build /bin/bash -c "
# Checkout head.
mkdir /project && cd /project
git init
git remote add origin /repo
git fetch --depth 1 origin HEAD
git checkout FETCH_HEAD

# Copy untracked and modified files, and remove deleted files, from our current repo.
cd /repo
{ git ls-files --others --exclude-standard ; git diff --name-only --diff-filter=TMAR HEAD ; } | rsync -a --files-from=- . /project
for F in \$(git ls-files --deleted); do rm /project/\$F > /dev/null 2>&1; done

cd /project
docker build -f bootstrap/$DOCKERFILE .
"