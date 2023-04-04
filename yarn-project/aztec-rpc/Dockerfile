FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY . .
# Build barretenberg.js to ensure wasm is in the dest folder.
WORKDIR /usr/src/yarn-project/barretenberg.js
RUN yarn build

# dependencies
WORKDIR /usr/src/yarn-project
COPY barretenberg.js barretenberg.js

RUN cd aztec-node && yarn build
WORKDIR /usr/src/yarn-project/aztec-rpc
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/aztec-rpc /usr/src/yarn-project/aztec-rpc
WORKDIR /usr/src/yarn-project/aztec-rpc
ENTRYPOINT ["yarn"]