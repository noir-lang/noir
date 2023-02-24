FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY public-client public-client
WORKDIR /usr/src/yarn-project/public-client
ARG ARG_COMMIT_HASH
ENV COMMIT_HASH=$ARG_COMMIT_HASH
RUN yarn build && yarn formatting && yarn test
# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
RUN apk update && apk add --no-cache llvm15-libs curl jq
COPY --from=builder /usr/src /usr/src
WORKDIR /usr/src/yarn-project/public-client
ENTRYPOINT ["yarn"]
CMD ["start"]
EXPOSE 80