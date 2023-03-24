FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY foundation foundation
COPY aztec.js aztec.js
RUN cd aztec.js && yarn build

COPY end-to-end end-to-end
WORKDIR /usr/src/yarn-project/end-to-end
RUN yarn build && yarn formatting

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
RUN apk update && apk add --no-cache curl jq bash
COPY --from=builder /usr/src/yarn-project /usr/src/yarn-project
WORKDIR /usr/src/yarn-project/end-to-end
ENTRYPOINT ["yarn", "test"]