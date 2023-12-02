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
FROM test-base as test-js
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
RUN ./scripts/test3.sh
RUN rm -rf /usr/src/noir/tooling/noir_js/test/noir_compiled_examples/assert_lt/target/debug_assert_lt.json
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