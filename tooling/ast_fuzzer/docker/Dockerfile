# syntax=docker/dockerfile:1

# Builder
FROM rust:1.85.0-slim-bookworm AS builder

RUN apt-get update && \
  apt-get install -y git && \
  rm -rf /var/lib/apt/lists/*


WORKDIR /noir

COPY . .

RUN --mount=type=cache,target=target \
  cargo install --locked --path tooling/nargo_cli


# Runner
FROM debian:bookworm-slim

RUN apt-get update && \
  apt-get install -y cvise && \
  rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/nargo /usr/local/bin/