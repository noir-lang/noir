use dep::protocol_types::point::Point;

/// Converts a public key to a byte array.
///
/// We don't serialize the point at infinity flag because this function is used in situations where we do not want
/// to waste the extra byte (encrypted log).
pub fn pub_key_to_bytes(pk: Point) -> [u8; 64] {
    assert(!pk.is_infinite, "Point at infinity is not a valid public key.");
    let mut result = [0 as u8; 64];
    let x_bytes = pk.x.to_be_bytes(32);
    let y_bytes = pk.y.to_be_bytes(32);
    for i in 0..32 {
        result[i] = x_bytes[i];
        result[i + 32] = y_bytes[i];
    }
    result
}
