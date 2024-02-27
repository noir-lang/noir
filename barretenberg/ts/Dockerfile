FROM aztecprotocol/barretenberg-wasm-linux-clang

FROM node:18.19.0-alpine
COPY --from=0 /usr/src/barretenberg /usr/src/barretenberg

# Create a standalone container that can run bb.js (and tests).
# We'll perform the build in a new, different directory, so the original directory can become the "published" package.
WORKDIR /usr/src/barretenberg/ts-build
# Leverage layer caching. Only re-install packages if these files change.
COPY .yarn .yarn
COPY package.json package.json
COPY yarn.lock yarn.lock
COPY .yarnrc.yml .yarnrc.yml
RUN yarn --immutable
COPY . .
RUN yarn formatting && SKIP_CPP_BUILD=1 yarn build
CMD ["yarn", "test"]

# We want to create a pure package, as would be published to npm, for consuming projects.
RUN yarn pack && tar zxf package.tgz && rm package.tgz && mv package ../ts