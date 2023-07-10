FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/barretenberg-x86_64-linux-clang-assert
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/bb.js

FROM rust:alpine AS atbbc
RUN apk update && apk add musl-dev
WORKDIR /usr/src/barretenberg/acir_tests
COPY acir-to-bberg-circuit acir-to-bberg-circuit
RUN cd acir-to-bberg-circuit && cargo build --release

FROM node:18-alpine
RUN apk update && apk add git bash curl
COPY --from=0 /usr/src/barretenberg/cpp/build /usr/src/barretenberg/cpp/build
COPY --from=1 /usr/src/barretenberg/ts /usr/src/barretenberg/ts
COPY --from=2 /usr/src/barretenberg/acir_tests/acir-to-bberg-circuit /usr/src/barretenberg/acir_tests/acir-to-bberg-circuit
WORKDIR /usr/src/barretenberg/acir_tests
COPY . .
RUN ./run_acir_tests.sh
RUN BB=../ts/dest/main.js ./run_acir_tests.sh