FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY kernel-simulator kernel-simulator
WORKDIR /usr/src/yarn-project/kernel-simulator
RUN yarn build && yarn formatting && yarn test

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/kernel-simulator /usr/src/yarn-project/kernel-simulator