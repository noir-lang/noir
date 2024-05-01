#!/bin/bash
set -e

# Modify the uid and gid of aztec-dev to match that of the host users.
[ -n "$LOCAL_GROUP_ID" ] && groupmod -g $LOCAL_GROUP_ID aztec-dev
[ -n "$LOCAL_USER_ID" ] && usermod -u $LOCAL_USER_ID aztec-dev &> /dev/null

# Find the group id of the docker socket, add aztec-dev to that group, or create the group and add aztec-dev.
if [ -S /var/run/docker.sock ]; then
    SOCKET_GID=$(stat -c %g /var/run/docker.sock)
    EXISTING_GROUP=$(getent group $SOCKET_GID | cut -d: -f1)
    if [ -z "$EXISTING_GROUP" ]; then
        # No existing group with that gid, so create one called 'docker' and add the user to it.
        groupadd -g $SOCKET_GID docker
        usermod -aG docker aztec-dev
    else
        # A group with the desired gid already exists, add the user to it.
        usermod -aG $EXISTING_GROUP aztec-dev
    fi
fi

exec /usr/sbin/gosu aztec-dev "$@"