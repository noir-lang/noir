// TODO: change this to be called Grumpkin Point
struct Point {
    x: Field,
    y: Field,
}

global POINT_SERIALIZED_LEN: Field = 2;

impl Point {
    pub fn zero() -> Self {
        Point {
            x: 0,
            y: 0,
        }
    }

    fn serialize(self) -> [Field; POINT_SERIALIZED_LEN] {
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
