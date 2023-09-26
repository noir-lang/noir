FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY . .

WORKDIR /usr/src/yarn-project/ethereum
RUN yarn build && yarn formatting

RUN yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/ /usr/src/
WORKDIR /usr/src/yarn-project/ethereum
ENTRYPOINT ["yarn"]