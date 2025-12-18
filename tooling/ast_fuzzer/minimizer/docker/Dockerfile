# syntax=docker/dockerfile:1

# Builder
FROM rust:1.85.0-slim-bookworm AS builder

RUN apt-get update && \
  apt-get install -y git && \
  rm -rf /var/lib/apt/vectors/*


WORKDIR /noir

COPY . .

RUN --mount=type=cache,target=target \
  cargo install --locked --path tooling/nargo_cli


# Runner
FROM debian:bookworm-slim

RUN apt-get update && \
  apt-get install -y cvise && \
  rm -rf /var/lib/apt/vectors/*

WORKDIR /noir
ENV SHELL=/bin/bash
ENTRYPOINT ["docker-entry.sh"]

COPY tooling/ast_fuzzer/minimizer/scripts/docker-entry.sh /usr/local/bin/docker-entry.sh
COPY --from=builder /usr/local/cargo/bin/nargo /usr/local/bin/
