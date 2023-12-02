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

FROM rust:1-slim-bookworm as test-base
RUN apt-get update && apt-get upgrade -y && apt-get install build-essential git -y
WORKDIR /usr/src/noir
COPY . .
RUN ./scripts/bootstrap_native.sh
ENV PATH="${PATH}:/usr/src/noir/target/release/"

FROM test-base as test-cargo
RUN apt-get install -y curl libc++-dev
RUN ./scripts/test_native.sh

# openssl libc++-dev libncurses5 curl jq npm nodejs
FROM test-base as js-test
RUN apt-get install pkg-config libssl-dev -y
RUN ./scripts/install_wasm-bindgen.sh
RUN apt-get install -y ca-certificates curl gnupg
RUN mkdir -p /etc/apt/keyrings
RUN curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg
RUN echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_20.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list
RUN apt-get update && apt-get install nodejs -y
RUN corepack enable
RUN yarn --immutable
RUN apt-get install -y jq
RUN yarn build
RUN yarn workspace @noir-lang/acvm_js test

RUN npx playwright install && npx playwright install-deps
RUN yarn workspace @noir-lang/acvm_js test:browser

RUN yarn workspace @noir-lang/noirc_abi test
RUN yarn workspace @noir-lang/noirc_abi test:browser
RUN yarn workspace @noir-lang/backend_barretenberg test
RUN yarn workspace @noir-lang/noir_js test
RUN yarn workspace @noir-lang/source-resolver test
RUN ./scripts/test.sh
RUN yarn workspace @noir-lang/noir_wasm test:node
RUN yarn workspace @noir-lang/noir_wasm test:browser
RUN ./scripts/test2.sh
RUN rm -rf /usr/src/noir/tooling/noir_codegen/test/assert_lt/target/debug_assert_lt.json
RUN yarn workspace @noir-lang/noir_codegen test
RUN apt-get install -y libc++-dev
RUN yarn test:integration


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

# FROM rust:alpine3.17 as test-js
# RUN apk update \
#     && apk upgrade \
#     && apk add --no-cache \
#         build-base \
#         pkgconfig \
#         openssl-dev \
#         npm \
#         yarn \
#         bash \
#         jq \
#         git
# # RUN apt-get update && apt-get upgrade -y && apt-get install build-essential git openssl libc++-dev libncurses5 curl -y
# WORKDIR /usr/src/noir
# COPY . .
# RUN ./scripts/install_wasm-bindgen.sh
# RUN export SOURCE_DATE_EPOCH=$(date +%s) && GIT_DIRTY=false && export GIT_COMMIT=$(git rev-parse --verify HEAD) && cargoExtraArgs="--features noirc_driver/aztec"

# RUN cargo build --features="noirc_driver/aztec" --release
# ENV PATH="${PATH}:/usr/src/noir/target/release/"
# RUN yarn && yarn build && yarn add playwright && yarn playwright install
# RUN yarn test

FROM node:20-bookworm-slim as test-js
RUN apt-get update && apt-get upgrade -y && apt-get install build-essential git openssl libc++-dev libncurses5 curl jq libssl-dev pkg-config -y
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
WORKDIR /usr/src/noir
COPY . .
ENV PATH="${PATH}:/root/.cargo/bin/"
RUN ./scripts/test_js_packages.sh

# FROM rust:1-slim-bookworm as test-js
# COPY --from=js-install /usr/local/bin /usr/local/bin
# RUN apt-get update && apt-get upgrade -y && apt-get install build-essential git openssl libc++-dev libncurses5 curl npm nodejs -y
# WORKDIR /usr/src/noir
# COPY . .
# RUN export SOURCE_DATE_EPOCH=$(date +%s) && GIT_DIRTY=false && export GIT_COMMIT=$(git rev-parse --verify HEAD)
# RUN cargo build --features="noirc_driver/aztec" --release
# RUN yarn && yarn build && yarn add playwright && yarn playwright install


# RUN export SOURCE_DATE_EPOCH=$(date +%s) && GIT_DIRTY=false && export GIT_COMMIT=$(git rev-parse --verify HEAD) && cargoExtraArgs="--features noirc_driver/aztec"
