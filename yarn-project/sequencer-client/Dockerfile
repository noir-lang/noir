FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY sequencer-client sequencer-client
WORKDIR /usr/src/yarn-project/sequencer-client
RUN yarn build && yarn formatting && yarn test

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/sequencer-client /usr/src/yarn-project/sequencer-client