# AST fuzz targets

This crate was created by `cargo fuzz init`. See more in https://rust-fuzz.github.io/book/cargo-fuzz/

Execute it with the following command:

```shell
cd tooling/ast_fuzzer
cargo +nightly fuzz run ast -- -runs=1000 -max_total_time=60
```