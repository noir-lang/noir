## setup
```
rustup install nightly
rustup default nightly
cargo install cargo-fuzz
```
## available
1) field
2) uint
3) branching
## fuzz
`cargo fuzz run uint --fuzz-dir .`

OR in 5 threads

`nohup cargo-fuzz run fuzz_target_1 -- -jobs=5 -workers=5 &`
