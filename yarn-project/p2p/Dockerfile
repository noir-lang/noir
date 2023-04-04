FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY p2p p2p
# dependencies
COPY circuits.js circuits.js
COPY ethereum.js ethereum.js
COPY foundation foundation
COPY l1-contracts l1-contracts
COPY l2-block l2-block
COPY tx tx

WORKDIR /usr/src/yarn-project/p2p
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/p2p /usr/src/yarn-project/p2p
WORKDIR /usr/src/yarn-project/p2p
ENTRYPOINT ["yarn"]