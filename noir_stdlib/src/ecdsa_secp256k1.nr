#[foreign(ecdsa_secp256k1)]
// docs:start:ecdsa_secp256k1
/// Verifies a ECDSA signature over the secp256k1 curve.
/// - inputs:
///     - x coordinate of public key as 32 bytes
///     - y coordinate of public key as 32 bytes
///     - the signature, as a 64 bytes array
///       The signature internally will be represented as `(r, s)`,
///       where `r` and `s` are fixed-sized big endian scalar values.
///       As the `secp256k1` has a 256-bit modulus, we have a 64 byte signature
///       while `r` and `s` will both be 32 bytes.
///       We expect `s` to be normalized. This means given the curve's order,
///       `s` should be less than or equal to `order / 2`.
///       This is done to prevent malleability.
///       For more context regarding malleability you can reference BIP 0062.
///     - the hash of the message, as a vector of bytes
/// - output: false for failure and true for success
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
