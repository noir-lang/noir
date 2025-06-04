#!/usr/bin/env bash

# Build a docker image that has `nargo` and `cvise` in it, to minimise Noir code.

ROOT_DIR=$(dirname $0)/../../..
DOCKER_FILE=$ROOT_DIR/tooling/ast_fuzzer/docker/Dockerfile

DOCKER_BUILDKIT=1 \
    docker build \
        -f $DOCKER_FILE \
        -t noir-minimizer \
        $ROOT_DIR