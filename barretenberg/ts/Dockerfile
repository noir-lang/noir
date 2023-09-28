FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/barretenberg-wasm-linux-clang

FROM node:18-alpine
COPY --from=0 /usr/src/barretenberg /usr/src/barretenberg
WORKDIR /usr/src/barretenberg/ts
# Leverage layer caching. Only re-install packages if these files change.
COPY .yarn .yarn
COPY package.json package.json
COPY yarn.lock yarn.lock
COPY .yarnrc.yml .yarnrc.yml
RUN yarn --immutable
COPY . .
RUN yarn formatting && SKIP_CPP_BUILD=1 yarn build
CMD ["yarn", "test"]
