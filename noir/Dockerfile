FROM rust:alpine3.17
RUN apk update \
    && apk upgrade \
    && apk add --no-cache \
        build-base
WORKDIR /usr/src/noir
COPY . .
RUN GIT_DIRTY=false \
    GIT_COMMIT=$(awk '/commit =/ {print $3}' .gitrepo) \
    cargo build --features="noirc_frontend/aztec" --release

FROM alpine:3.17
COPY --from=0 /usr/src/noir/target/release/nargo /usr/bin/nargo
WORKDIR /project
ENTRYPOINT ["/usr/bin/nargo"]