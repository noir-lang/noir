FROM aztecprotocol/noir as noir
FROM aztecprotocol/avm-transpiler as transpiler
FROM --platform=linux/amd64 aztecprotocol/barretenberg-x86_64-linux-clang as barretenberg

FROM ubuntu:lunar as builder
RUN apt-get update && apt-get install -y parallel nodejs npm jq curl
RUN npm install --global yarn
# Copy in bb
COPY --from=barretenberg /usr/src/barretenberg/cpp/build/bin/bb /usr/src/barretenberg/cpp/build/bin/bb
# Copy in nargo
COPY --from=noir /usr/src/noir/noir-repo/target/release/nargo /usr/src/noir/noir-repo/target/release/nargo
ENV PATH="/usr/src/noir/noir-repo/target/release:${PATH}"
# Copy in transpiler
COPY --from=transpiler /usr/src/avm-transpiler/target/release/avm-transpiler /usr/src/avm-transpiler/target/release/avm-transpiler
ENV PATH="/usr/src/avm-transpiler/target/release:${PATH}"
# Copy in noir projects
WORKDIR /usr/src/noir-projects
COPY . .
# Build
WORKDIR /usr/src/noir-projects/noir-contracts
RUN ./bootstrap.sh
WORKDIR /usr/src/noir-projects/noir-protocol-circuits
RUN ./bootstrap.sh
WORKDIR /usr/src/noir-projects/aztec-nr
RUN nargo compile --silence-warnings

FROM scratch
COPY --from=builder /usr/src/noir-projects /usr/src/noir-projects