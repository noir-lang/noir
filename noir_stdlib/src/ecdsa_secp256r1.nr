/// Verifies an ECDSA signature over the secp256r1 curve, returning `true` if it is valid.
///
/// The public key is given by its `x` and `y` coordinates, 32 bytes each. The signature is the
/// 64 bytes `r || s`, with `r` and `s` 32 bytes each. `message_hash` is the 32-byte hash of the
/// signed message and is used as given: the function does not hash the message, and a hash that is
/// not less than the curve's order is reduced modulo the order, as in the ECDSA specification. All
/// values are big-endian.
///
/// An invalid signature makes the function return `false`, never fail, so the caller must check
/// the result, typically with `assert`. The result is also `false` if `r` or `s` is zero, if the
/// public key is not a point on the curve, or if `s` is greater than `order / 2`: only the
/// low-`s` form of a signature is accepted, to prevent malleability (see BIP 0062).
///
/// Example:
///
/// ```noir
/// let message_hash = [
///     84, 112, 91, 163, 186, 175, 219, 223, 186, 140, 95, 154, 112, 247, 168, 155, 238, 152, 217, 6,
///     181, 62, 49, 7, 77, 167, 186, 236, 220, 13, 169, 173,
/// ];
/// let public_key_x = [
///     85, 15, 71, 16, 3, 243, 223, 151, 195, 223, 80, 106, 199, 151, 246, 114, 31, 177, 161, 251,
///     123, 143, 111, 131, 210, 36, 73, 138, 101, 200, 142, 36,
/// ];
/// let public_key_y = [
///     19, 96, 147, 215, 1, 46, 80, 154, 115, 113, 92, 189, 11, 0, 163, 204, 15, 244, 181, 192, 27,
///     63, 250, 25, 106, 177, 251, 50, 112, 54, 184, 230,
/// ];
/// let signature = [
///     44, 112, 168, 208, 132, 182, 43, 252, 92, 224, 54, 65, 202, 249, 247, 42, 212, 218, 140, 129,
///     191, 230, 236, 148, 135, 187, 94, 27, 239, 98, 161, 50, 24, 173, 158, 226, 158, 175, 53, 31,
///     220, 80, 241, 82, 12, 66, 94, 155, 144, 138, 7, 39, 139, 67, 176, 236, 123, 135, 39, 120,
///     193, 78, 7, 132,
/// ];
///
/// let valid = std::ecdsa_secp256r1::verify_signature(public_key_x, public_key_y, signature, message_hash);
/// assert(valid);
///
/// // A different message hash doesn't verify.
/// let mut other_hash = message_hash;
/// other_hash[0] = 0;
/// assert(!std::ecdsa_secp256r1::verify_signature(public_key_x, public_key_y, signature, other_hash));
/// ```
// docs:start:ecdsa_secp256r1
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
