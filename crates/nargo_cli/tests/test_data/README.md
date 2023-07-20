# Noir Test Programs

A suite of Noir programs that have been pre-compiled to ACIR bytecode and witness data. These can be fed into simulators and backends to ensure they produce expected proofs.

The witness data is output in "flat" format, which at the time of writing requires nargo to be compiled with the "flat_witness" feature flag enabled.

```
nix develop # (hopefully soon optional)
cargo install --path=crates/nargo_cli --no-default-features --features=flat_witness
```

Then to recompile:

```
./rebuild.sh
```
