FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY @types @types
COPY foundation foundation
WORKDIR /usr/src/yarn-project/foundation
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/foundation /usr/src/yarn-project/foundation
WORKDIR /usr/src/yarn-project/foundation
ENTRYPOINT ["yarn"]