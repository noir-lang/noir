FROM --platform=linux/amd64 aztecprotocol/bb.js as bb.js
FROM --platform=linux/amd64 aztecprotocol/noir-packages as noir-packages

FROM node:18.19.0 as builder

# Copy in portalled packages.
# We need them because faucet is part of the workspace, and the workspace depends on them. 🤷‍♂️
COPY --from=bb.js /usr/src/barretenberg/ts /usr/src/barretenberg/ts
COPY --from=noir-packages /usr/src/noir/packages /usr/src/noir/packages

WORKDIR /usr/src/yarn-project
COPY . .
RUN yarn workspaces focus @aztec/aztec-faucet
WORKDIR /usr/src/yarn-project/aztec-faucet
RUN yarn tsc -b
RUN yarn workspaces focus @aztec/aztec-faucet --production && yarn cache clean

# Create minimal image.
FROM ubuntu:lunar
ARG COMMIT_TAG=""
ENV COMMIT_TAG=$COMMIT_TAG
RUN apt update && apt install -y nodejs && rm -rf /var/lib/apt/lists/* && apt-get clean
COPY --from=builder /usr/src/yarn-project /usr/src/yarn-project
ENTRYPOINT ["node", "--no-warnings", "/usr/src/yarn-project/aztec-faucet/dest/bin/index.js"]
EXPOSE 8080
