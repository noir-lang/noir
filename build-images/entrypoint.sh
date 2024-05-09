#!/bin/bash
set -e

# Modify the uid and gid of aztec-dev to match that of the host users.
[ -n "$LOCAL_GROUP_ID" ] && groupmod -g $LOCAL_GROUP_ID aztec-dev
[ -n "$LOCAL_USER_ID" ] && usermod -u $LOCAL_USER_ID aztec-dev &> /dev/null

/usr/local/share/docker-init.sh &> /dev/null

exec /usr/sbin/gosu aztec-dev "$@"