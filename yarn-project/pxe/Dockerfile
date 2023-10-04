FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project AS yarn-project

ARG COMMIT_TAG=""

# Start a new image as we need arch specific.
FROM node:18-alpine as builder
RUN apk add jq
COPY --from=yarn-project /usr/src/ /usr/src/

# Update pxe version if COMMIT_TAG has been used
WORKDIR /usr/src/yarn-project/pxe
RUN if [[ -n "${COMMIT_TAG}" ]]; then \
  jq --arg v ${COMMIT_TAG} '.version = $v' package.json > _temp && mv _temp package.json; \
  fi

# Productionify. See comment in yarn-project-base/Dockerfile.
RUN yarn cache clean && yarn workspaces focus --production

# Create final, minimal size image.
FROM node:18-alpine
COPY --from=builder /usr/src/ /usr/src/
WORKDIR /usr/src/yarn-project/pxe
ENTRYPOINT ["yarn"]
CMD [ "start" ]
EXPOSE 8081
