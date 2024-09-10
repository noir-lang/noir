---
title: poseidon
---

# Module `std::hash::poseidon`

### Methods

### Methods

## config

```rust
fn config<let T: u32, let N: u32, let X: u32>(t: Field, rf: u8, rp: u8, alpha: Field, round_constants: [Field; N], mds: [[Field; T]; T], presparse_mds: [[Field; T]; T], sparse_mds: [Field; X]) -> PoseidonConfig<T, N, X>
```

## permute

```rust
fn permute<let T: u32, let N: u32, let X: u32>(pos_conf: PoseidonConfig<T, N, X>, mut state: [Field; T]) -> [Field; T]
```

