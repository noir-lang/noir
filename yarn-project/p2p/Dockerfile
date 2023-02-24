FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY p2p p2p
WORKDIR /usr/src/yarn-project/p2p
RUN yarn build && yarn formatting && yarn test

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/p2p /usr/src/yarn-project/p2p