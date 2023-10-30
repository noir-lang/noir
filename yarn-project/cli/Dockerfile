FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project AS yarn-project

# Start a new image as we need arch specific.
FROM node:18-alpine as builder
ARG COMMIT_TAG=""
RUN apk add jq
COPY --from=yarn-project /usr/src/ /usr/src/

WORKDIR /usr/src/yarn-project/cli
RUN if [[ -n "${COMMIT_TAG}" ]]; then \
  jq --arg v ${COMMIT_TAG} '.version = $v' package.json > _temp && mv _temp package.json; \
  fi

# Productionify. See comment in yarn-project-base/Dockerfile.
RUN yarn cache clean && yarn workspaces focus --production

# Create final, minimal size image.
FROM node:18-alpine

COPY --from=builder /usr/src/ /usr/src/

ENV XDG_CACHE_HOME /cache
RUN mkdir /cache && chmod 777 /cache
VOLUME [ "/cache" ]

# run as non-root user
RUN addgroup -S aztec && adduser -S aztec -G aztec
USER aztec

ENV NODE_OPTIONS "--no-warnings"

ENTRYPOINT ["node", "/usr/src/yarn-project/cli/dest/bin/index.js"]
