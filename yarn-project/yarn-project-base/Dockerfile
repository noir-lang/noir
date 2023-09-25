# This base dockerfile is for caching the workspace dependencies and running workspace checks.
# The dockerignore file ensures the context only contains package.json and tsconfig.json files.
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/circuits-wasm-linux-clang as circuits
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/l1-contracts as contracts

FROM node:18-alpine AS builder
RUN apk update && apk add --no-cache build-base git python3 curl bash jq sed
WORKDIR /usr/src/yarn-project
# The dockerignore file ensures the context only contains package.json and tsconfig.json files.
# WARNING: boxes is an exception! See dockerignore file.
COPY . .
# Install all workspace dependencies.
RUN yarn --immutable
# If everything's worked properly, we should no longer need access to the network.
RUN echo "enableNetwork: false" >> .yarnrc.yml


# Yarn devs won't provide an extremely simple and useful feature of pruning dev dependencies from the local cache:
# https://github.com/yarnpkg/berry/issues/1789
#
# To work around this, we will construct a global cache from the local cache using hard links (requires a hacky rename).
# This adds a 0 byte layer to the docker image. Genius!
#
# When we build a downstream docker image, we:
# - Do the build in a named "builder" stage.
# - Erase the local cache with a `yarn cache clean`. Files will remain in global cache due to hard link.
# - Do a `yarn workspaces focus --production` to install production dependencies from the global cache, to .yarn/cache
# - Create a final stage from desired output, stripping away the global cache layer.
RUN /bin/bash -c '\
  [ -d /root/.yarn/berry/cache ] && exit 0; \
  cd .yarn/cache && \
  mkdir -p /root/.yarn/berry/cache && \
  for F in *; do \
  [[ $F =~ (.*-) ]] && ln $F /root/.yarn/berry/cache/${BASH_REMATCH[1]}8.zip; \
  done'

# Check package.json inheritence and tsconfig project references.
RUN yarn prepare:check

# Bring in circuits wasms.
COPY --from=circuits /usr/src/circuits/cpp/build-wasm/bin /usr/src/circuits/cpp/build-wasm/bin

# Generate L1 contract TypeScript artifacts.
COPY --from=contracts /usr/src/l1-contracts/out /usr/src/l1-contracts/out
RUN cd l1-artifacts && ./scripts/generate-artifacts.sh
