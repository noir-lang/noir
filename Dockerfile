FROM rust:bullseye
WORKDIR /usr/src/noir
COPY . .
RUN ./scripts/bootstrap_native.sh

# When running the container, mount the users home directory to same location.
FROM ubuntu:focal
# Install Tini as nargo doesn't handle signals properly.
# Install git as nargo needs it to clone.
RUN apt-get update && apt-get install -y git tini && rm -rf /var/lib/apt/lists/* && apt-get clean
COPY --from=0 /usr/src/noir/target/release/nargo /usr/src/noir/target/release/nargo
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/src/noir/target/release/nargo"]
