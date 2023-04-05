FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

# (1) project
COPY . .

# (2) build
WORKDIR /usr/src/yarn-project/circuits.js
RUN yarn build && yarn formatting

# (3) copy bb.js wasm binaries
RUN cp /usr/src/circuits/cpp/build-wasm/bin/aztec3-circuits.wasm /usr/src/yarn-project/circuits.js/dest/wasm/aztec3-circuits.wasm

# (3.5) test
RUN yarn test

# (4) Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

# (5) set up entry point
FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project /usr/src/yarn-project
WORKDIR /usr/src/yarn-project/circuits.js
ENTRYPOINT ["yarn", "test"]
