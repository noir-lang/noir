use dep::std::cmp::Eq;

global GRUMPKIN_POINT_SERIALIZED_LEN: Field = 2;

struct GrumpkinPoint {
    x: Field,
    y: Field,
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
        Self {
            x: 0,
            y: 0,
        }
    }

    pub fn is_zero(self) -> bool {
        (self.x == 0) & (self.y == 0)
    }

    fn serialize(self) -> [Field; GRUMPKIN_POINT_SERIALIZED_LEN] {
        [self.x, self.y]
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
