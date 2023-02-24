FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY wallet wallet
WORKDIR /usr/src/yarn-project/wallet
RUN yarn build && yarn formatting && yarn test

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/wallet /usr/src/yarn-project/wallet