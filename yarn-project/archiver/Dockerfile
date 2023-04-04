FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY archiver archiver
# dependencies
COPY ethereum.js ethereum.js
COPY foundation foundation
COPY circuits.js circuits.js
COPY l1-contracts l1-contracts
COPY l2-block l2-block

WORKDIR /usr/src/yarn-project/archiver
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/archiver /usr/src/yarn-project/archiver
WORKDIR /usr/src/yarn-project/archiver
ENTRYPOINT ["yarn"]