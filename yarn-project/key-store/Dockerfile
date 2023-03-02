FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY key-store key-store
WORKDIR /usr/src/yarn-project/key-store
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/key-store /usr/src/yarn-project/key-store
WORKDIR /usr/src/yarn-project/key-store
ENTRYPOINT ["yarn"]