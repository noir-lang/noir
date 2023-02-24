FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-base AS builder
COPY data-archiver data-archiver
WORKDIR /usr/src/yarn-project/data-archiver
RUN yarn build && yarn formatting && yarn test

FROM alpine:latest
COPY --from=builder /usr/src/yarn-project/data-archiver /usr/src/yarn-project/data-archiver