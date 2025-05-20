## Constraint counts

Current count: 428,240.

## Compile:

`time nargo compile`

## Get gate count:

Install bbup: `curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash`
The version of bb compatible with nargo 0.33.0 is `bbup --version 0.48.0`

`time bb gates --scheme client_ivc -b ./target/blob.json`

## Generate, then serve a new flamegraph, after compiling:

<!-- `~/packages/noir/noir-repo/target/release/noir-profiler gates-flamegraph --artifact-path ./target/blob.json --backend-path ~/.bb/bb --output ./flamegraph -- -h && python3 -m http.server --directory "./flamegraph" 3000` -->

`~/packages/noir/noir-repo/target/release/noir-profiler gates --artifact-path ./target/blob.json --backend-path ~/.bb/bb --output ./flamegraph --backend-gates-command "gates" --scheme client_ivc --include_gates_per_opcode && python3 -m http.server --directory "./flamegraph" 3000`

## To serve an existing flamegraph:

`python3 -m http.server --directory "./flamegraph" 3000`
