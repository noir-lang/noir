FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY . .

# Manually run bb so we get the wasm in dest
WORKDIR /usr/src/yarn-project/barretenberg.js
RUN yarn build

# Manually run circuits so we get the wasm in dest
WORKDIR /usr/src/yarn-project/circuits.js
RUN yarn build

WORKDIR /usr/src/yarn-project/p2p
RUN yarn build && yarn formatting

RUN yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/p2p /usr/src/yarn-project/p2p
WORKDIR /usr/src/yarn-project/p2p
ENTRYPOINT ["yarn"]