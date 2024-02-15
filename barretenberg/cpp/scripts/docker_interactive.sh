# Script to enter a docker shell.
# This comes in two flavors, ubuntu and alpine.
# This mounts the current folder into the image, allowing for in-docker development.
# You then need to run cmake commands fresh to build. 
# Ideally you can combine with a fresh clone of the repo to keep it persistently around, or move around build dirs as needed.
# This is useful for debugging issues like bb being slow on alpine or running into errors on a specific version of gcc.
# usage: ./docker_interactive.sh ubuntu or ./docker_interactive.sh alpine

# Enter script directory.
set -eu
cd $(dirname $0)
# allows for 'alpine', 'ubuntu' or 'msan.ubuntu'
ENVIRONMENT_KIND="${1:-alpine}"
DOCKERFILE="../dockerfiles/interactive/Dockerfile.$ENVIRONMENT_KIND"
docker build -t "env-$ENVIRONMENT_KIND" -f "$DOCKERFILE" .
docker run -it --mount type=bind,source=$(pwd)/..,target=/usr/src/barretenberg/cpp "env-$ENVIRONMENT_KIND"
