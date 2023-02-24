FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY acir-simulator acir-simulator
WORKDIR /usr/src/yarn-project/acir-simulator
RUN yarn build && yarn formatting && yarn test

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/acir-simulator /usr/src/yarn-project/acir-simulator