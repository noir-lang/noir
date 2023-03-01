FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY private-client private-client
WORKDIR /usr/src/yarn-project/private-client
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/private-client /usr/src/yarn-project/private-client
WORKDIR /usr/src/yarn-project/private-client
ENTRYPOINT ["yarn"]