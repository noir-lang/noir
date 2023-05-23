FROM ghcr.io/foundry-rs/foundry:nightly-c331b5eeee1b4151ef7354a081667e2d770b37f5 as builder

# Required for foundry
RUN apk update && apk add git

WORKDIR /usr/src/l1-contracts

# Install foundry
COPY . .
RUN git init

# Install deps
RUN forge install --no-commit \
  https://github.com/foundry-rs/forge-std \
  https://github.com/openzeppelin/openzeppelin-contracts

# Run build and tests
RUN forge clean && forge fmt --check && forge build && forge test

WORKDIR /usr/src/l1-contracts
