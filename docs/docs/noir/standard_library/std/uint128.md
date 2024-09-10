---
title: uint128
---

# Module `std::uint128`

### Methods

#### from_u64s_le

```noir
fn from_u64s_le(lo: u64, hi: u64) -> U128
```

#### from_u64s_be

```noir
fn from_u64s_be(hi: u64, lo: u64) -> U128
```

#### zero

```noir
fn zero() -> U128
```

#### one

```noir
fn one() -> U128
```

#### from_le_bytes

```noir
fn from_le_bytes(bytes: [u8; 16]) -> U128
```

#### to_be_bytes

```noir
fn to_be_bytes(self) -> [u8; 16]
```

#### to_le_bytes

```noir
fn to_le_bytes(self) -> [u8; 16]
```

#### from_hex

```noir
fn from_hex<let N: u32>(hex: str<N>) -> U128
```

