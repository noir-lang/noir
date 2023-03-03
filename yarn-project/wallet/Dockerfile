FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY wallet wallet
WORKDIR /usr/src/yarn-project/wallet
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/wallet /usr/src/yarn-project/wallet
WORKDIR /usr/src/yarn-project/wallet
ENTRYPOINT ["yarn"]