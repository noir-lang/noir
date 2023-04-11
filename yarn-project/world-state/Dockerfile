FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY . .

WORKDIR /usr/src/yarn-project/world-state
RUN yarn build && yarn formatting

RUN yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

# set up entry point
FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/world-state /usr/src/yarn-project/world-state
WORKDIR /usr/src/yarn-project/world-state
ENTRYPOINT ["yarn", "test"]