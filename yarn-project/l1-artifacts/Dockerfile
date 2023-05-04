FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY . .

WORKDIR /usr/src/yarn-project/l1-artifacts
RUN yarn build

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/l1-artifacts /usr/src/yarn-project/l1-artifacts
WORKDIR /usr/src/yarn-project/l1-artifacts
ENTRYPOINT ["yarn"]