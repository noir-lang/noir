FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY kernel-simulator kernel-simulator
WORKDIR /usr/src/yarn-project/kernel-simulator
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/kernel-simulator /usr/src/yarn-project/kernel-simulator
WORKDIR /usr/src/yarn-project/kernel-simulator
ENTRYPOINT ["yarn"]