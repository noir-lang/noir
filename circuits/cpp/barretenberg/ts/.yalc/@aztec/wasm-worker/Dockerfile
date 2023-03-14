FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY wasm-worker wasm-worker
WORKDIR /usr/src/yarn-project/wasm-worker
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/wasm-worker /usr/src/yarn-project/wasm-worker
WORKDIR /usr/src/yarn-project/wasm-worker
ENTRYPOINT ["yarn"]