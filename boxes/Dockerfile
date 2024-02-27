# Builds the boxes (they were copied into yarn-project-base so the cli can unbox).
# Produces a container that can be run to test a specific box. See docker-compose.yml.
FROM aztecprotocol/aztec AS aztec
FROM aztecprotocol/noir as noir
FROM aztecprotocol/noir-projects as noir-projects

# We need yarn. Start fresh container.
FROM node:18.19.0
RUN apt update && apt install netcat-openbsd
COPY --from=aztec /usr/src /usr/src
COPY --from=noir /usr/src/noir/target/release/nargo /usr/src/noir/target/release/nargo
COPY --from=noir-projects /usr/src/noir-projects/aztec-nr /usr/src/noir-projects/aztec-nr
WORKDIR /usr/src/boxes
ENV AZTEC_NARGO=/usr/src/noir/target/release/nargo
ENV AZTEC_CLI=/usr/src/yarn-project/cli/aztec-cli-dest
RUN yarn && yarn build
ENTRYPOINT ["/bin/sh", "-c"]
