FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/circuits-x86_64-linux-clang
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/circuits-wasm-linux-clang
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/l1-contracts

FROM node:18-alpine
RUN apk update && apk add --no-cache build-base git python3 curl bash jq
COPY --from=0 /usr/src/circuits/barretenberg/cpp/srs_db/ignition /usr/src/circuits/barretenberg/cpp/srs_db/ignition
COPY --from=0 /usr/src/circuits/barretenberg/cpp/srs_db/download_ignition.sh /usr/src/circuits/barretenberg/cpp/srs_db/download_ignition.sh
COPY --from=0 /usr/src/circuits/build/bin /usr/src/circuits/build/bin
COPY --from=1 /usr/src/circuits/build/bin /usr/src/circuits/build-wasm/bin
COPY --from=2 /usr/src/l1-contracts /usr/src/l1-contracts
WORKDIR /usr/src/yarn-project
# We only want to copy the package.json's, to ensure we only rebuild this image if project dependencies changed.
COPY acir-simulator/package.json acir-simulator/package.json
COPY aztec-cli/package.json aztec-cli/package.json
COPY aztec.js/package.json aztec.js/package.json
COPY data-archiver/package.json data-archiver/package.json
COPY ethereum.js/package.json ethereum.js/package.json
COPY kernel-simulator/package.json kernel-simulator/package.json
COPY p2p/package.json p2p/package.json
COPY private-client/package.json private-client/package.json
COPY prover-client/package.json prover-client/package.json
COPY public-client/package.json public-client/package.json
COPY sequencer-client/package.json sequencer-client/package.json
COPY wallet/package.json wallet/package.json

# All workspaces use the linting config, so always include it.
COPY eslint-config eslint-config
COPY prettier-config prettier-config
COPY .pnp.cjs .pnp.loader.mjs .yarnrc.yml package.json yarn.lock ./
COPY .yarn .yarn
# Although we're attempting to be "zero-install", in practice we still need to build arch specific packages.
RUN yarn --immutable
# If everything's worked properly, we should no longer need access to the network.
RUN echo "enableNetwork: false" >> .yarnrc.yml

# Yarn devs won't provide an extremely simple and useful feature of pruning dev dependencies from the local cache:
# https://github.com/yarnpkg/berry/issues/1789
#
# To work around this, we construct a global cache from the local cache using hard links (requires a hacky rename).
# When we build an upstream docker image, we:
# - Do the build.
# - Erase the local cache with a `yarn cache clean`. Files remain in global cache due to hard link.
# - Do a `yarn workspaces focus --production` to install production dependencies from the global cache, to .yarn/cache
# - A final stage of the build strips away the global cache.
RUN /bin/bash -c '\
[ -d /root/.yarn/berry/cache ] && exit 0; \
cd .yarn/cache && \
mkdir -p /root/.yarn/berry/cache && \
for F in *; do \
  [[ $F =~ (.*-) ]] && ln $F /root/.yarn/berry/cache/${BASH_REMATCH[1]}8.zip; \
done'