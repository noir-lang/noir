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
