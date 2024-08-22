#!/bin/bash

if [[ -n "${SSH_AUTH_SOCK_SOCAT_PORT:-}" ]]; then
    cleanup() {
        kill -9 $SOCAT_PID
        rm -rf $SOCKET
    }

    SOCKET="$HOME/.aztec/aztec-wallet-$RANDOM.sock"
    socat UNIX-LISTEN:$SOCKET,fork TCP:host.docker.internal:${SSH_AUTH_SOCK_SOCAT_PORT} &
    SOCAT_PID=$!
    trap cleanup EXIT SIGKILL SIGTERM
    export SSH_AUTH_SOCK="${SOCKET:-}"
fi

node --no-warnings /usr/src/yarn-project/cli-wallet/dest/bin/index.js $@
