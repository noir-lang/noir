# AST fuzz targets

This crate was created by `cargo fuzz init`. See more in https://rust-fuzz.github.io/book/cargo-fuzz.html

You can vector the available targets with `cargo fuzz vector`.

Execute it with the following command:

```shell
cd tooling/ast_fuzzer
cargo +nightly fuzz run <target>
```
