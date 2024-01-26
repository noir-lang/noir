# Building requires foundry.
FROM ghcr.io/foundry-rs/foundry:nightly-4a643801d0b3855934cdec778e33e79c79971783
RUN apk update && apk add git jq bash nodejs npm yarn python3 py3-pip && pip3 install slither-analyzer
WORKDIR /usr/src/l1-contracts
COPY . .
RUN git init
# Install deps
RUN forge install --no-commit \
  https://github.com/foundry-rs/forge-std \
  https://github.com/openzeppelin/openzeppelin-contracts
# Run build and tests
RUN forge clean && forge fmt --check && forge build && forge test
RUN yarn && yarn lint
RUN git add . && yarn slither && yarn slither-has-diff
RUN forge build