FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

# Copy in the whole of yarn-project.
COPY . .

# Build circuits.js to ensure wasm is in the dest folder.
WORKDIR /usr/src/yarn-project/circuits.js
RUN yarn build

# Build end-to-end.
WORKDIR /usr/src/yarn-project/end-to-end
RUN yarn build && yarn formatting

# Prune dev dependencies. See comment in base image.
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

# Create final, minimal size image.
FROM node:18-alpine
RUN apk update && apk add --no-cache curl jq bash
COPY --from=builder /usr/src /usr/src
WORKDIR /usr/src/yarn-project/end-to-end
ENTRYPOINT ["yarn", "test"]