# AST fuzz targets

This crate was created by `cargo fuzz init`. See more in https://rust-fuzz.github.io/book/cargo-fuzz/

You can list the available targets with `cargo fuzz list`.

Execute it with the following command:

```shell
cd tooling/ast_fuzzer
cargo +nightly fuzz run <target>
```