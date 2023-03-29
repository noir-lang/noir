FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY . .
RUN cd aztec.js && yarn build
RUN cd noir-contracts && yarn build
WORKDIR /usr/src/yarn-project/end-to-end
RUN yarn formatting

# copy bb.js wasm binaries
WORKDIR /usr/src/yarn-project/barretenberg.js
RUN cp /usr/src/aztec3-circuits/cpp/build-wasm/bin/aztec3-circuits.wasm dest/wasm/barretenberg.wasm

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
RUN apk update && apk add --no-cache curl jq bash
COPY --from=builder /usr/src/yarn-project /usr/src/yarn-project
WORKDIR /usr/src/yarn-project/end-to-end
ENTRYPOINT ["yarn", "test"]