#!/bin/bash
set -eu

cd $(dirname $0)

# On linux we need to perform uid/gid alignment to ensure files modified on the host have the correct owner.
# The entrypoint.sh script picks up these environment variables and adjusts the aztec-dev user accordingly.
# This isn't necessary on mac.
if [[ "$OSTYPE" == "linux"* ]]; then
  ID_ARGS="-e LOCAL_USER_ID=$(id -u) -e LOCAL_GROUP_ID=$(id -g)"
fi

docker run \
  -ti --rm \
  --hostname devbox \
  -e SSH_CONNECTION=' ' \
  ${ID_ARGS:-} \
  -w/workspaces/aztec-packages \
  -v$PWD/..:/workspaces/aztec-packages \
  -vdevbox-home:/home/aztec-dev \
  -v$HOME/.ssh/id_rsa:/home/aztec-dev/.ssh/id_rsa:ro \
  -v/var/run/docker.sock:/var/run/docker.sock \
  aztecprotocol/devbox
