# Bytecode Size Comparison

These scripts can be used to compare the bytecode size of circuits in `aztec-packages` between two different versions of `nargo`.

## Compiling contracts

Run these commands to compile Noir protocol circuits and contracts in `aztec-packages` after rebuilding `nargo`:

```shell
cargo build -p nargo_cli --release
./target/release/nargo --program-dir ../aztec-packages/noir-projects/noir-protocol-circuits compile --force --silence-warnings --skip-underconstrained-check
./target/release/nargo --program-dir ../aztec-packages/noir-projects/noir-contracts compile --force --silence-warnings --skip-underconstrained-check
```

## Baseline

Record the baseline bytecode size with before switching to other implementations:
```shell
./scripts/bytecode-sizes/print-bytecode-size.sh ../aztec-packages > ./scripts/bytecode-sizes/baseline.jsonl
```

## Alternative

After making some changes to `nargo`, compile the contracts again with the commands above, then run the following
commands to record a new measurement, and compare it against the baseline recorded earlier.

```shell
BASELINE=baseline
ALTERNATIVE=alternative
./scripts/bytecode-sizes/print-bytecode-size.sh ../aztec-packages \
    > ./scripts/bytecode-sizes/$ALTERNATIVE.jsonl
./scripts/bytecode-sizes/compare-bytecode-size.sh \
    ./scripts/bytecode-sizes/$BASELINE.jsonl \
    ./scripts/bytecode-sizes/$ALTERNATIVE.jsonl \
    > ./scripts/bytecode-sizes/$BASELINE-vs-$ALTERNATIVE.jsonl
./scripts/bytecode-sizes/plot-bytecode-size.sh \
    ./scripts/bytecode-sizes/$BASELINE-vs-$ALTERNATIVE.jsonl
```

You can look at the impact in `./scripts/bytecode-sizes/$BASELINE-vs-$ALTERNATIVE.png`.
