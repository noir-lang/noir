FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY world-state world-state
# dependencies
COPY foundation foundation
COPY circuits.js circuits.js
COPY l2-block l2-block
COPY merkle-tree merkle-tree
# TODO(AD) why do we need to build dependencies? dont tsc project refs solve this?
WORKDIR /usr/src/yarn-project/foundation
RUN yarn build
WORKDIR /usr/src/yarn-project/circuits.js
RUN yarn build
WORKDIR /usr/src/yarn-project/l2-block
RUN yarn build
WORKDIR /usr/src/yarn-project/merkle-tree
RUN yarn build
WORKDIR /usr/src/yarn-project/world-state
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/world-state /usr/src/yarn-project/world-state
WORKDIR /usr/src/yarn-project/world-state
ENTRYPOINT ["yarn"]