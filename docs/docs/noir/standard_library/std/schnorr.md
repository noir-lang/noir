# Module `std::schnorr`

## verify_signature

```noir
fn verify_signature<let N: u32>(public_key_x: Field, public_key_y: Field, signature: [u8; 64], message: [u8; N]) -> bool
```

## verify_signature_slice

```noir
fn verify_signature_slice(public_key_x: Field, public_key_y: Field, signature: [u8; 64], message: [u8]) -> bool
```

