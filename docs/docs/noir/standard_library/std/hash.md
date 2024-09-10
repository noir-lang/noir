---
title: hash
---

# Module `std::hash`

## Public exports

 - `use sha256::digest`
 - `use sha256::sha256`
 - `use sha256::sha256_compression`
 - `use sha256::sha256_var`
### Methods

## blake2s

```rust
fn blake2s<let N: u32>(input: [u8; N]) -> [u8; 32]
```

## blake3

```rust
fn blake3<let N: u32>(input: [u8; N]) -> [u8; 32]
```

## pedersen_commitment

```rust
fn pedersen_commitment<let N: u32>(input: [Field; N]) -> EmbeddedCurvePoint
```

## pedersen_hash_with_separator

```rust
fn pedersen_hash_with_separator<let N: u32>(input: [Field; N], separator: u32) -> Field
```

## pedersen_hash

```rust
fn pedersen_hash<let N: u32>(input: [Field; N]) -> Field
```

## hash_to_field

```rust
fn hash_to_field(inputs: [Field]) -> Field
```

## keccak256

```rust
fn keccak256<let N: u32>(input: [u8; N], message_size: u32) -> [u8; 32]
```

## poseidon2_permutation

```rust
fn poseidon2_permutation<let N: u32>(_input: [Field; N], _state_length: u32) -> [Field; N]
```

