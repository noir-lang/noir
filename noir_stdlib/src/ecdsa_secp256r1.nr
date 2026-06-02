/// Verifier for ECDSA Secp256r1 signatures.
// docs:start:ecdsa_secp256r1
/// Verifies a ECDSA signature over the secp256r1 curve.
/// - inputs:
///     - x coordinate of public key as 32 bytes
///     - y coordinate of public key as 32 bytes
///     - the signature, as a 64 bytes array
///       The signature internally will be represented as `(r, s)`,
///       where `r` and `s` are fixed-sized big endian scalar values.
///       As the `secp256r1` has a 256-bit modulus, we have a 64 byte signature
///       while `r` and `s` will both be 32 bytes.
///       We expect `s` to be normalized. This means given the curve's order,
///       `s` should be less than or equal to `order / 2`.
///       This is done to prevent malleability.
///       For more context regarding malleability you can reference BIP 0062.
///     - the hash of the message, as a vector of bytes
/// - output: false for failure and true for success
pub fn verify_signature(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8; 32],
) -> bool
// docs:end:ecdsa_secp256r1
{
    _verify_signature(public_key_x, public_key_y, signature, message_hash, true)
}

#[foreign(ecdsa_secp256r1)]
fn _verify_signature(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8; 32],
    predicate: bool,
) -> bool {}

mod tests {
    #[test]
    fn verify() {
        let mut hashed_message: [u8; 32] = [
            84, 112, 91, 163, 186, 175, 219, 223, 186, 140, 95, 154, 112, 247, 168, 155, 238, 152,
            217, 6, 181, 62, 49, 7, 77, 167, 186, 236, 220, 13, 169, 173,
        ];

        let pub_key_x: [u8; 32] = [
            85, 15, 71, 16, 3, 243, 223, 151, 195, 223, 80, 106, 199, 151, 246, 114, 31, 177, 161,
            251, 123, 143, 111, 131, 210, 36, 73, 138, 101, 200, 142, 36,
        ];
        let pub_key_y: [u8; 32] = [
            19, 96, 147, 215, 1, 46, 80, 154, 115, 113, 92, 189, 11, 0, 163, 204, 15, 244, 181, 192,
            27, 63, 250, 25, 106, 177, 251, 50, 112, 54, 184, 230,
        ];
        let signature: [u8; 64] = [
            44, 112, 168, 208, 132, 182, 43, 252, 92, 224, 54, 65, 202, 249, 247, 42, 212, 218, 140,
            129, 191, 230, 236, 148, 135, 187, 94, 27, 239, 98, 161, 50, 24, 173, 158, 226, 158,
            175, 53, 31, 220, 80, 241, 82, 12, 66, 94, 155, 144, 138, 7, 39, 139, 67, 176, 236, 123,
            135, 39, 120, 193, 78, 7, 132,
        ];
        let valid_signature =
            super::verify_signature(pub_key_x, pub_key_y, signature, hashed_message);
        assert(valid_signature);

        hashed_message[0] = 0;
        let valid_signature =
            super::verify_signature(pub_key_x, pub_key_y, signature, hashed_message);
        assert(!valid_signature);
    }
}
