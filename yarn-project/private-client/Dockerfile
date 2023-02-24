FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY private-client private-client
WORKDIR /usr/src/yarn-project/private-client
RUN yarn build && yarn formatting && yarn test

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/private-client /usr/src/yarn-project/private-client