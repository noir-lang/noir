pub use dep::std::embedded_curve_ops::EmbeddedCurvePoint as Point;
use crate::{hash::poseidon2_hash, traits::{Deserialize, Empty, Hash, Packable, Serialize}};

pub global POINT_LENGTH: u32 = 3;

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
        Point { x: 0, y: 0, is_infinite: false }
    }
}

impl Deserialize<POINT_LENGTH> for Point {
    fn deserialize(serialized: [Field; POINT_LENGTH]) -> Point {
        Point { x: serialized[0], y: serialized[1], is_infinite: serialized[2] as bool }
    }
}
// TODO(#11356): use compact representation here.
impl Packable<POINT_LENGTH> for Point {
    fn pack(self) -> [Field; POINT_LENGTH] {
        self.serialize()
    }

    fn unpack(packed: [Field; POINT_LENGTH]) -> Self {
        Self::deserialize(packed)
    }
}
