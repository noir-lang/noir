FROM rust:bullseye

WORKDIR /usr/src
COPY ./avm-transpiler ./avm-transpiler
COPY ./noir ./noir

WORKDIR /usr/src/avm-transpiler
RUN apt-get update && apt-get install -y git
RUN ./scripts/bootstrap_native.sh

FROM ubuntu:focal
COPY --from=0 /usr/src/avm-transpiler/target/release/avm-transpiler /usr/src/avm-transpiler/target/release/avm-transpiler
ENTRYPOINT ["sh", "-c"]
