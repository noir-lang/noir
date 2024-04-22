FROM --platform=linux/amd64 aztecprotocol/bb.js as bb.js
FROM --platform=linux/amd64 aztecprotocol/noir-packages as noir-packages
FROM --platform=linux/amd64 aztecprotocol/l1-contracts as contracts
FROM --platform=linux/amd64 aztecprotocol/noir-projects as noir-projects
FROM aztecprotocol/noir as noir
FROM --platform=linux/amd64 aztecprotocol/barretenberg-x86_64-linux-clang as barretenberg

FROM node:18.19.0 as builder
RUN apt update && apt install -y jq curl perl && rm -rf /var/lib/apt/lists/* && apt-get clean

# Copy in portalled packages.
COPY --from=bb.js /usr/src/barretenberg/ts /usr/src/barretenberg/ts
COPY --from=noir-packages /usr/src/noir/packages /usr/src/noir/packages
COPY --from=contracts /usr/src/l1-contracts /usr/src/l1-contracts
COPY --from=noir-projects /usr/src/noir-projects /usr/src/noir-projects
# We want the native ACVM and BB binaries
COPY --from=noir /usr/src/noir/noir-repo/target/release/acvm /usr/src/noir/noir-repo/target/release/acvm
COPY --from=barretenberg /usr/src/barretenberg/cpp/build/bin/bb /usr/src/barretenberg/cpp/build/bin/bb

WORKDIR /usr/src/yarn-project
COPY . .

# We install a symlink to yarn-project's node_modules at a location that all portalled packages can find as they
# walk up the tree as part of module resolution. The supposedly idiomatic way of supporting module resolution
# correctly for portalled packages, is to use --preserve-symlinks when running node.
# This does kind of work, but jest doesn't honor it correctly, so this seems like a neat workaround.
# Also, --preserve-symlinks causes duplication of portalled instances such as bb.js, and breaks the singleton logic
# by initialising the module more than once. So at present I don't see a viable alternative.
RUN ln -s /usr/src/yarn-project/node_modules /usr/src/node_modules

# TODO: Replace puppeteer with puppeteer-core to avoid this.
ENV PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true

RUN ./bootstrap.sh
RUN yarn workspaces focus @aztec/aztec --production && yarn cache clean

# TODO: Use release-please to update package.json directly, and remove this!
# It's here to ensure the image rebuilds if the commit tag changes (as the content hash won't).
# ARG COMMIT_TAG=""
# RUN ./scripts/version_packages.sh

# We no longer need these.
RUN rm -rf /usr/src/noir-projects /usr/src/l1-contracts

# Create minimal size image.
FROM node:18.19.1-slim
ARG COMMIT_TAG=""
ENV COMMIT_TAG=$COMMIT_TAG
COPY --from=builder /usr/src /usr/src
WORKDIR /usr/src/yarn-project
ENTRYPOINT ["yarn"]
