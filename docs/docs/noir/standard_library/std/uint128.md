---
title: uint128
---

# Module `std::uint128`

### Methods

#### from_u64s_le

```rust
fn from_u64s_le(lo: u64, hi: u64) -> U128
```

#### from_u64s_be

```rust
fn from_u64s_be(hi: u64, lo: u64) -> U128
```

#### zero

```rust
fn zero() -> U128
```

#### one

```rust
fn one() -> U128
```

#### from_le_bytes

```rust
fn from_le_bytes(bytes: [u8; 16]) -> U128
```

#### to_be_bytes

```rust
fn to_be_bytes(self) -> [u8; 16]
```

#### to_le_bytes

```rust
fn to_le_bytes(self) -> [u8; 16]
```

#### from_hex

```rust
fn from_hex<let N: u32>(hex: str<N>) -> U128
```

