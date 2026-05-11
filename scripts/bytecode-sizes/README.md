# Bytecode Size Comparison

These scripts can be used to compare the bytecode size of circuits in `aztec-packages` between two different versions of `nargo`.

## Build `nargo`

```shell
cargo build -p nargo_cli --release
```

## Baseline format

Pick one of the existing serialization formats, for example:

```shell
BASELINE=msgpack-compact
export NOIR_SERIALIZATION_FORMAT=$BASELINE
```

## Compiling contracts

Run these commands to compile Noir protocol circuits and contracts in `aztec-packages` after rebuilding `nargo`:

```shell
./target/release/nargo --program-dir ../aztec-packages/noir-projects/noir-protocol-circuits compile --force --silence-warnings --skip-underconstrained-check --skip-brillig-constraints-check
./target/release/nargo --program-dir ../aztec-packages/noir-projects/noir-contracts compile --force --silence-warnings --skip-underconstrained-check --skip-brillig-constraints-check
```

## Baseline measurement

Record the baseline bytecode size with before switching to other implementations:
```shell
./scripts/bytecode-sizes/print-bytecode-size.sh ../aztec-packages > ./scripts/bytecode-sizes/$BASELINE.jsonl
```

## Alternative

After making some changes to `nargo`, or setting an alternative format, compile the contracts again with the commands above. For example here we just set a different format as an alternative, before running the commands again:

```shell
ALTERNATIVE=msgpack-tagged
export NOIR_SERIALIZATION_FORMAT=$ALTERNATIVE
```

Once the contracts are recompiled, run the following commands to record the new measurement, then compare it against the baseline recorded earlier:

```shell
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
