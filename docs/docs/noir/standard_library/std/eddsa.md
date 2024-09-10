---
title: eddsa
---

# Module `std::eddsa`

## eddsa_poseidon_verify

```rust
fn eddsa_poseidon_verify(pub_key_x: Field, pub_key_y: Field, signature_s: Field, signature_r8_x: Field, signature_r8_y: Field, message: Field) -> bool
```

## eddsa_verify

```rust
fn eddsa_verify<H>(pub_key_x: Field, pub_key_y: Field, signature_s: Field, signature_r8_x: Field, signature_r8_y: Field, message: Field) -> bool
    where H: Hasher,
          H: Default
```

## eddsa_to_pub

```rust
fn eddsa_to_pub(secret: Field) -> (Field, Field)
```

