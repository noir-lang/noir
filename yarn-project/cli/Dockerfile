FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project AS yarn-project

# Need new arch specific image.
FROM node:18-alpine as builder
COPY --from=yarn-project /usr/src /usr/src
ARG COMMIT_TAG=""

WORKDIR /usr/src/yarn-project/cli
RUN if [[ -n "${COMMIT_TAG}" ]]; then \
  jq --arg v ${COMMIT_TAG} '.version = $v' package.json > _temp && mv _temp package.json; \
  fi

# Productionify. See comment in yarn-project-base/Dockerfile.
RUN yarn workspaces focus --production && yarn cache clean && rm -rf ../**/src

# Create final, arch specific, minimal size image.
FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project /usr/src/yarn-project
COPY --from=builder /usr/src/barretenberg/ts/package /usr/src/barretenberg/ts/package
COPY --from=builder /usr/src/noir/packages /usr/src/noir/packages

ENV XDG_CACHE_HOME /cache
RUN mkdir /cache && chmod 777 /cache
VOLUME [ "/cache" ]

RUN corepack enable

# run as non-root user
RUN addgroup -S aztec && adduser -S aztec -G aztec
USER aztec

ENV NODE_OPTIONS="--no-warnings --preserve-symlinks"
ENTRYPOINT ["node", "/usr/src/yarn-project/cli/dest/bin/index.js"]