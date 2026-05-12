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

Make some changes to `nargo`, or set an alternative format, for example:

```shell
ALTERNATIVE=msgpack-tagged
export NOIR_SERIALIZATION_FORMAT=$ALTERNATIVE
```

Then compile the contracts again with the commands above.

Once the contracts are recompiled, run the following commands to record the new measurement, and compare it against the baseline recorded earlier:

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

Two scatter plots are produced:

* `./scripts/bytecode-sizes/$BASELINE-vs-$ALTERNATIVE-compressed.png` —
  ratio of the gzipped + base64-encoded bytecode that actually ships in
  the artifact (what end users see).
* `./scripts/bytecode-sizes/$BASELINE-vs-$ALTERNATIVE-uncompressed.png` —
  ratio of the pre-gzip msgpack bytes. Isolates wire-format cost from
  gzip's compression ratio.

The two can diverge: a denser pre-gzip encoding may compress slightly
worse than a more verbose one (gzip rewards repetition), so a small
post-compression regression on a particular program doesn't necessarily
mean the wire format itself grew.
