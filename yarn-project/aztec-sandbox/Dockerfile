FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project AS yarn-project

# Need new arch specific image.
FROM node:18-alpine as builder
COPY --from=yarn-project /usr/src /usr/src
ARG COMMIT_TAG=""

# Update pxe version if COMMIT_TAG has been used.
WORKDIR /usr/src/yarn-project/pxe
RUN if [[ -n "${COMMIT_TAG}" ]]; then \
  jq --arg v ${COMMIT_TAG} '.version = $v' package.json > _temp && mv _temp package.json; \
  fi

# Update sandbox version if COMMIT_TAG has been used.
WORKDIR /usr/src/yarn-project/aztec-sandbox
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
# Just until weird source-resolver bug fixed.
COPY --from=yarn-project /usr/src/noir/compiler /usr/src/noir/compiler
WORKDIR /usr/src/yarn-project/aztec-sandbox
ENV NODE_OPTIONS=--preserve-symlinks
ENTRYPOINT ["yarn"]
CMD [ "start" ]
EXPOSE 8079 8080
