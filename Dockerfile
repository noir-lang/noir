FROM rust:alpine3.17 as build
RUN apk update \
    && apk upgrade \
    && apk add --no-cache \
        build-base \
        bash \
        git
WORKDIR /usr/src/noir
COPY . .
RUN ./scripts/bootstrap_native.sh

# When running the container, mount the current working directory to /project.
FROM alpine:3.17 as production
COPY --from=build /usr/src/noir/target/release/nargo /usr/src/noir/target/release/nargo
WORKDIR /project
ENTRYPOINT ["/usr/src/noir/target/release/nargo"]

FROM rust:1-slim-bookworm as test
RUN apt-get update && apt-get upgrade -y && apt-get install git openssl -y
WORKDIR /usr/src/noir
COPY . .
RUN export SOURCE_DATE_EPOCH=$(date +%s) && GIT_DIRTY=false && export GIT_COMMIT=$(git rev-parse --verify HEAD)
RUN cargo build --features="noirc_driver/aztec" --release
RUN cargo test --workspace --locked --release
