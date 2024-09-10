# Module `std::ecdsa_secp256k1`

## verify_signature

```noir
fn verify_signature<let N: u32>(public_key_x: [u8; 32], public_key_y: [u8; 32], signature: [u8; 64], message_hash: [u8; N]) -> bool
```

## verify_signature_slice

```noir
fn verify_signature_slice(public_key_x: [u8; 32], public_key_y: [u8; 32], signature: [u8; 64], message_hash: [u8]) -> bool
```

