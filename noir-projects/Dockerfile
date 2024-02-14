FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/noir as noir
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/avm-transpiler as transpiler
FROM ubuntu:lunar

# Copy in nargo
COPY --from=noir /usr/src/noir/target/release/nargo /usr/src/noir/target/release/nargo
# Copy in transpiler
COPY --from=transpiler /usr/src/avm-transpiler/target/release/avm-transpiler /usr/src/avm-transpiler/target/release/avm-transpiler

WORKDIR /usr/src/noir-projects
# Copy in noir projects
COPY . .
# Build
WORKDIR /usr/src/noir-projects/noir-contracts
RUN ./scripts/compile.sh && ./scripts/transpile.sh && ../../noir/target/release/nargo test --silence-warnings

WORKDIR /usr/src/noir-projects/noir-protocol-circuits
RUN cd src && ../../../noir/target/release/nargo compile --silence-warnings && ../../../noir/target/release/nargo test --silence-warnings

WORKDIR /usr/src/noir-projects/aztec-nr
RUN ../../noir/target/release/nargo compile --silence-warnings && ../../noir/target/release/nargo test --silence-warnings
