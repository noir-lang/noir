// TODO(https://github.com/noir-lang/noir/issues/7711): Switch the regular comments to doc comments once they do not trigger a warning anymore
#[foreign(ecdsa_secp256k1)]
// docs:start:ecdsa_secp256k1
// The signature internally will be represented as `(r, s)`,
// where `r` and `s` are fixed-sized big endian scalar values.
// As the `secp256k1` has a 256-bit modulus, we have a 64 byte signature
// while `r` and `s` will both be 32 bytes.
// We expect `s` to be normalized. This means given an the curve's order,
// `s` should be less than or equal to `order / 2`.
// This is done to prevent malleability.
// For more context regarding malleability you can reference BIP 0062.
pub fn verify_signature<let N: u32>(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8; N],
) -> bool
// docs:end:ecdsa_secp256k1
{}

#[foreign(ecdsa_secp256k1)]
// docs:start:ecdsa_secp256k1_slice
pub fn verify_signature_slice(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8],
) -> bool
// docs:end:ecdsa_secp256k1_slice
{}
