FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY . .

WORKDIR /usr/src/yarn-project/noir-contracts
RUN yarn build && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/noir-contracts /usr/src/yarn-project/noir-contracts
WORKDIR /usr/src/yarn-project/noir-contracts
ENTRYPOINT ["yarn"]