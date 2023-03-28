FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY tx tx
WORKDIR /usr/src/yarn-project/tx
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/tx /usr/src/yarn-project/tx
WORKDIR /usr/src/yarn-project/tx
ENTRYPOINT ["yarn"]