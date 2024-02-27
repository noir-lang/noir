FROM aztecprotocol/noir as noir
FROM aztecprotocol/avm-transpiler as transpiler

FROM ubuntu:lunar AS builder
# Copy in nargo
COPY --from=noir /usr/src/noir/target/release/nargo /usr/src/noir/target/release/nargo
# Copy in transpiler
COPY --from=transpiler /usr/src/avm-transpiler/target/release/avm-transpiler /usr/src/avm-transpiler/target/release/avm-transpiler
# Copy in noir projects
WORKDIR /usr/src/noir-projects
COPY . .
# Build
WORKDIR /usr/src/noir-projects/noir-contracts
RUN ./scripts/compile.sh && ./scripts/transpile.sh && ../../noir/target/release/nargo test --silence-warnings
WORKDIR /usr/src/noir-projects/noir-protocol-circuits
RUN cd src && ../../../noir/target/release/nargo compile --silence-warnings && ../../../noir/target/release/nargo test --silence-warnings
WORKDIR /usr/src/noir-projects/aztec-nr
RUN ../../noir/target/release/nargo compile --silence-warnings && ../../noir/target/release/nargo test --silence-warnings

FROM scratch
COPY --from=builder /usr/src/noir-projects /usr/src/noir-projects