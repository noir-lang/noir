FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY aztec.js aztec.js
WORKDIR /usr/src/yarn-project/aztec.js
RUN yarn build && yarn formatting && yarn test

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/aztec.js /usr/src/yarn-project/aztec.js
WORKDIR /usr/src/yarn-project/aztec.js
ENTRYPOINT ["yarn"]