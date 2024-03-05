FROM alpine:3.18
RUN apk update \
    && apk upgrade \
    && apk add --no-cache \
        build-base \
        clang16 \
        openmp-dev \
        cmake \
        ninja \
        git \
        curl \
        perl

WORKDIR /usr/src/barretenberg/cpp

COPY ./cpp .
# Build everything to ensure everything builds. All tests will be run from the result of this build.
RUN cmake --preset clang16  && cmake --build --preset clang16 --target solidity_key_gen solidity_proof_gen

FROM docker.io/frolvlad/alpine-glibc:alpine-3.17_glibc-2.34 as builder
RUN apk update && apk add git curl build-base openmp-dev bash

COPY --from=0 /usr/src/barretenberg/cpp/build/bin /usr/src/barretenberg/cpp/build/bin
COPY --from=0 /usr/src/barretenberg/cpp/srs_db /usr/src/barretenberg/cpp/srs_db
WORKDIR /usr/src/barretenberg/sol
COPY ./sol .

# Copy forge binary directly from foundry
COPY --from=ghcr.io/foundry-rs/foundry:latest /usr/local/bin/forge /usr/local/bin/forge

RUN cd ../cpp/srs_db && ./download_ignition.sh 3 && cd ../../sol

RUN ./scripts/init.sh

# TestBase is excluded as it is just boilerplate
RUN forge test --no-match-contract TestBase