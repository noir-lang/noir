FROM ubuntu:focal as builder

# Required for foundry
RUN apt-get update && apt install -y git curl

WORKDIR /usr/src/contracts

# Install foundry
RUN git init
COPY . .
RUN ./scripts/install_foundry.sh
ENV PATH="/usr/src/contracts/.foundry/bin:${PATH}"

# Install deps
RUN forge install --no-commit \
  https://github.com/foundry-rs/forge-std \
  https://github.com/openzeppelin/openzeppelin-contracts
ENV MAINNET_RPC_URL='https://mainnet.infura.io/v3/9928b52099854248b3a096be07a6b23c'

# Run build and tests
RUN forge clean && forge build && forge test

WORKDIR /usr/src/contracts