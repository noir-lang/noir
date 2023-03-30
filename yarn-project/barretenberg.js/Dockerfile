FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY foundation foundation
COPY barretenberg.js barretenberg.js
WORKDIR /usr/src/yarn-project/barretenberg.js
RUN yarn build && yarn formatting && yarn test

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/barretenberg.js /usr/src/yarn-project/barretenberg.js