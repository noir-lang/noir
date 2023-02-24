FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY prover-client prover-client
WORKDIR /usr/src/yarn-project/prover-client
RUN yarn build && yarn formatting && yarn test

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/prover-client /usr/src/yarn-project/prover-client