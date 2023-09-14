FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder

COPY . .

WORKDIR /usr/src/yarn-project/boxes/private-token
RUN yarn build && yarn formatting

# this feels wrong
RUN yarn cache clean
RUN yarn workspaces focus --production > /dev/null

FROM node:18-alpine
COPY --from=builder /usr/src/yarn-project/boxes/private-token /usr/src/yarn-project/boxes/private-token
WORKDIR /usr/src/yarn-project/boxes/private-token
ENTRYPOINT ["yarn"] 