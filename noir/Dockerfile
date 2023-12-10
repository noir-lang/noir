FROM rust:bookworm
WORKDIR /usr/src/noir
COPY . .
RUN ./scripts/bootstrap_native.sh

# When running the container, mount the users home directory to /root
FROM ubuntu:lunar
COPY --from=0 /usr/src/noir/target/release/nargo /usr/src/noir/target/release/nargo
WORKDIR /root
ENTRYPOINT ["/usr/src/noir/target/release/nargo"]