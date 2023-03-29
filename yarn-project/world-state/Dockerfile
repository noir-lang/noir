FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY world-state world-state
# dependencies
COPY foundation foundation
COPY barretenberg.js barretenberg.js
COPY l1-contracts l1-contracts
COPY circuits.js circuits.js
COPY l2-block l2-block
COPY merkle-tree merkle-tree

RUN cp /usr/src/aztec3-circuits/cpp/build-wasm/bin/aztec3-circuits.wasm dest/wasm/barretenberg.wasm

WORKDIR /usr/src/yarn-project/world-state
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/world-state /usr/src/yarn-project/world-state
WORKDIR /usr/src/yarn-project/world-state
ENTRYPOINT ["yarn"]