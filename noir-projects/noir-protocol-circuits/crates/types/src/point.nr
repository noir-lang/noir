use dep::std::embedded_curve_ops::EmbeddedCurvePoint as Point;
use crate::{traits::{Empty, Hash, Serialize}, hash::poseidon2_hash};

global POINT_LENGTH: Field = 3;

impl Serialize<POINT_LENGTH> for Point {
    fn serialize(self: Self) -> [Field; POINT_LENGTH] {
        [self.x, self.y, self.is_infinite as Field]
    }
}

impl Hash for Point {
    fn hash(self) -> Field {
        poseidon2_hash(self.serialize())
    }
}

impl Empty for Point {
    /// Note: Does not return a valid point on curve - instead represents an empty/"unpopulated" point struct (e.g.
    /// empty/unpopulated value in an array of points).
    fn empty() -> Self {
        Point {
            x: 0,
            y: 0,
            is_infinite: false
        }
    }
}

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
