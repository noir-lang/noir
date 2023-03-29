FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

# (1) project and (2) dependencies
COPY . .

# (3) build
WORKDIR /usr/src/yarn-project/end-to-end
RUN yarn build && yarn formatting
# (4) copy bb.js wasm binaries
RUN cp /usr/src/aztec3-circuits/cpp/build-wasm/bin/aztec3-circuits.wasm /usr/src/yarn-project/barretenberg.js/dest/wasm/barretenberg.wasm

# (5) Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

# (6) set up entry point
FROM node:18-alpine
RUN apk update && apk add --no-cache curl jq bash
COPY --from=builder /usr/src/yarn-project /usr/src/yarn-project
WORKDIR /usr/src/yarn-project/
RUN yarn install
WORKDIR /usr/src/yarn-project/end-to-end
ENTRYPOINT ["yarn", "test"]