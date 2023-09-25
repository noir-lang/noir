# Linting requires node.
FROM node:18-alpine
RUN apk update && apk add --no-cache build-base git python3 curl bash jq
WORKDIR /usr/src/l1-contracts
COPY . .
RUN yarn && yarn lint

# Building requires foundry.
FROM ghcr.io/foundry-rs/foundry:nightly-c331b5eeee1b4151ef7354a081667e2d770b37f5
# Required for foundry
RUN apk update && apk add git
WORKDIR /usr/src/l1-contracts
COPY . .
RUN git init
# Install deps
RUN forge install --no-commit \
  https://github.com/foundry-rs/forge-std \
  https://github.com/openzeppelin/openzeppelin-contracts
# Run build and tests
RUN forge clean && forge fmt --check && forge build && forge test