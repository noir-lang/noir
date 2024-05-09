#!/bin/bash
set -eu

cd $(dirname $0)
hostname=$(hostname)

# Define next hostname based on this hostname for nesting. devbox, devbox1, etc.
if [[ $hostname == "devbox" ]]; then
  hostname="devbox1"
elif [[ $hostname =~ ^devbox([0-9]+)$ ]]; then
    num_suffix="${BASH_REMATCH[1]}"
    new_num=$((num_suffix + 1))
    hostname="devbox$new_num"
else
    hostname="devbox"
fi

# On linux we need to perform uid/gid alignment to ensure files modified on the host have the correct owner.
# The entrypoint.sh script picks up these environment variables and adjusts the aztec-dev user accordingly.
# This isn't necessary on mac.
if [[ "$OSTYPE" == "linux"* ]]; then
  ID_ARGS="-e LOCAL_USER_ID=$(id -u) -e LOCAL_GROUP_ID=$(id -g)"
fi

if docker ps -a --format '{{.Names}}' | grep -q '^aztec-devbox$'; then
  # Container already exists. Exec into a new shell.
  docker exec -ti --user aztec-dev aztec-devbox /bin/zsh
else
  # We override the docker config dir to ensure we don't conflict with vscodes dev-container.
  # They share the same home dir, but vscode will add some credentials config that it needs to its docker config.
  docker run \
    -ti --rm \
    --name aztec-devbox \
    --hostname $hostname \
    -e SSH_CONNECTION=' ' \
    -e DOCKER_CONFIG=/home/aztec-dev/.docker-devbox \
    ${ID_ARGS:-} \
    -w/workspaces/aztec-packages \
    -v$PWD/..:/workspaces/aztec-packages \
    -vdevbox-home:/home/aztec-dev \
    -vdevbox-var-lib-docker:/var/lib/docker \
    -v$HOME/.ssh/id_rsa:/home/aztec-dev/.ssh/id_rsa:ro \
    --privileged \
    aztecprotocol/devbox:1.0
fi