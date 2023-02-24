FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY aztec-cli aztec-cli
WORKDIR /usr/src/yarn-project/aztec-cli
RUN yarn build && yarn formatting && yarn test

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/aztec-cli /usr/src/yarn-project/aztec-cli