use crate::traits::{Serialize, Deserialize};
use dep::std::cmp::Eq;

global GRUMPKIN_POINT_SERIALIZED_LEN: Field = 2;

// TODO(https://github.com/noir-lang/noir/issues/4931)
struct GrumpkinPoint {
    x: Field,
    y: Field,
}

impl Serialize<GRUMPKIN_POINT_SERIALIZED_LEN> for GrumpkinPoint {
    fn serialize(self) -> [Field; GRUMPKIN_POINT_SERIALIZED_LEN] {
        [self.x, self.y]
    }
}

impl Deserialize<GRUMPKIN_POINT_SERIALIZED_LEN> for GrumpkinPoint {
    fn deserialize(serialized: [Field; GRUMPKIN_POINT_SERIALIZED_LEN]) -> Self {
        Self {
            x: serialized[0],
            y: serialized[1],
        }
    }
}

impl Eq for GrumpkinPoint {
    fn eq(self, point: GrumpkinPoint) -> bool {
        (point.x == self.x) & (point.y == self.y)
    }
}

impl GrumpkinPoint {
    pub fn new(x: Field, y: Field) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    pub fn is_zero(self) -> bool {
        (self.x == 0) & (self.y == 0)
    }

    // TODO(David): Would be quite careful here as (0,0) is not a point
    // on the curve. A boolean flag may be the better approach here,
    // would also cost less constraints. It seems like we don't need to 
    // group arithmetic either. 
    fn assert_is_zero(self) {
        assert(self.x == 0);
        assert(self.y == 0);
    }
}
