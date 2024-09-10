---
title: std
---

# Crate `std`

## println

```rust
fn println<T>(input: T)
```

## print

```rust
fn print<T>(input: T)
```

## verify_proof

```rust
fn verify_proof<let N: u32, let M: u32, let K: u32>(verification_key: [Field; N], proof: [Field; M], public_inputs: [Field; K], key_hash: Field)
```

## verify_proof_with_type

```rust
fn verify_proof_with_type<let N: u32, let M: u32, let K: u32>(verification_key: [Field; N], proof: [Field; M], public_inputs: [Field; K], key_hash: Field, proof_type: u32)
```

## assert_constant

```rust
fn assert_constant<T>(x: T)
```

## static_assert

```rust
fn static_assert<let N: u32>(predicate: bool, message: str<N>)
```

## wrapping_add

```rust
fn wrapping_add<T>(x: T, y: T) -> T
```

## wrapping_sub

```rust
fn wrapping_sub<T>(x: T, y: T) -> T
```

## wrapping_mul

```rust
fn wrapping_mul<T>(x: T, y: T) -> T
```

## as_witness

```rust
fn as_witness(x: Field)
```

