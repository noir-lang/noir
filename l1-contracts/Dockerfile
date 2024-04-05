# Building requires foundry.
FROM ubuntu:lunar

RUN apt update && apt install curl git jq bash nodejs npm python3.11-full python3-pip -y

# Use virtualenv, do not try to use pipx, it's not working.
# RUN python3 -m venv /root/.venv
# RUN /root/.venv/bin/pip3 install slither-analyzer==0.10.0 slitherin==0.5.0
RUN curl -L https://foundry.paradigm.xyz | bash

# Set env variables for foundry and venv
ENV PATH="${PATH}:/root/.foundry/bin:/root/.venv/bin"
RUN foundryup --version nightly-de33b6af53005037b463318d2628b5cfcaf39916

WORKDIR /usr/src/l1-contracts
COPY . .
RUN git init
RUN forge clean && forge fmt --check && forge build && forge test --no-match-contract UniswapPortalTest

RUN npm install --global yarn
RUN yarn && yarn lint

# RUN git add . && yarn slither && yarn slither-has-diff
RUN forge build

FROM scratch
COPY --from=0 /usr/src/l1-contracts/ /usr/src/l1-contracts/
