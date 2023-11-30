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

FROM build as test
RUN cargo test --workspace --locked --release
