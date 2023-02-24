FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY ethereum.js ethereum.js
WORKDIR /usr/src/yarn-project/ethereum.js
RUN yarn build && yarn formatting && yarn test

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/ethereum.js /usr/src/yarn-project/ethereum.js