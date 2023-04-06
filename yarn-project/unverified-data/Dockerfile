FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY unverified-data unverified-data
# dependencies
COPY foundation foundation
WORKDIR /usr/src/yarn-project/unverified-data
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/unverified-data /usr/src/yarn-project/unverified-data
WORKDIR /usr/src/yarn-project/unverified-data
ENTRYPOINT ["yarn"]