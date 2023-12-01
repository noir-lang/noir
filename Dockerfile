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
RUN apt-get update && apt-get upgrade -y && apt-get install build-essential git openssl libc++-dev libncurses5 curl -y
WORKDIR /usr/src/noir
COPY . .
RUN export SOURCE_DATE_EPOCH=$(date +%s) && GIT_DIRTY=false && export GIT_COMMIT=$(git rev-parse --verify HEAD)
RUN cargo build --features="noirc_driver/aztec" --release
RUN cargo test --workspace --locked --release

FROM rust:alpine3.17 as test-alpine
RUN apk update \
    && apk upgrade \
    && apk add --no-cache \
        build-base \
        pkgconfig \
        openssl-dev \
        npm \
        yarn \
        bash \
        jq \
        git \
        curl
WORKDIR /usr/src/noir
COPY . .
RUN export SOURCE_DATE_EPOCH=$(date +%s) && GIT_DIRTY=false && export GIT_COMMIT=$(git rev-parse --verify HEAD)
RUN cargo build --features="noirc_driver/aztec" --release
RUN cargo test --workspace --locked --release

FROM rust:alpine3.17 as test-js
RUN apk update \
    && apk upgrade \
    && apk add --no-cache \
        build-base \
        pkgconfig \
        openssl-dev \
        npm \
        yarn \
        bash \
        jq \
        git
# RUN apt-get update && apt-get upgrade -y && apt-get install build-essential git openssl libc++-dev libncurses5 curl -y
WORKDIR /usr/src/noir
COPY . .
RUN ./scripts/install_wasm-bindgen.sh
RUN export SOURCE_DATE_EPOCH=$(date +%s) && GIT_DIRTY=false && export GIT_COMMIT=$(git rev-parse --verify HEAD) && cargoExtraArgs="--features noirc_driver/aztec"

RUN cargo build --features="noirc_driver/aztec" --release
ENV PATH="${PATH}:/usr/src/noir/target/release/"
RUN yarn && yarn build && yarn add playwright && yarn playwright install
RUN yarn test

# RUN export SOURCE_DATE_EPOCH=$(date +%s) && GIT_DIRTY=false && export GIT_COMMIT=$(git rev-parse --verify HEAD) && cargoExtraArgs="--features noirc_driver/aztec"
