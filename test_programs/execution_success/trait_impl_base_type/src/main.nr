trait fieldable {
    fn to_field(self) -> field;
}

impl fieldable for u32 {
    fn to_field(self) -> field {
        let res = self as field;
        res * 3    
    }
}

impl fieldable for [u32; 3] {
    fn to_field(self) -> field {
        let res = self[0] + self[1] + self[2];
        res as field
    }
}

impl fieldable for bool {
    fn to_field(self) -> field {
        if self {
            14
        } else {
            3
        }
    }
}

impl fieldable for (u32, bool) {
    fn to_field(self) -> field {
        if self.1 {
            self.0 as field
        } else {
            32
        }
    }
}

impl fieldable for field {
    fn to_field(self) -> field {
        self
    }
}

impl fieldable for str<6> {
    fn to_field(self) -> field {
        6
    }
}

impl fieldable for () {
    fn to_field(self) -> field {
        0
    }
}

type Point2D = [field; 2];
type Point2DAlias = Point2D;

impl fieldable for Point2DAlias {
    fn to_field(self) -> field {
        self[0] + self[1]
    }
}

impl fieldable for fmtstr<14, (field, field)> {
    fn to_field(self) -> field {
        52
    }
}

impl fieldable for fn(u32) -> u32 {
    fn to_field(self) -> field {
        self(10) as field
    }
}

fn some_func(x: u32) -> u32 {
    x * 2 - 3
}

impl fieldable for u64 {
    fn to_field(self) -> field {
        66 as field
    }
}
// x = 15
fn main(x: u32) {
    assert(x.to_field() == 15);
    let arr: [u32; 3] = [3, 5, 8];
    assert(arr.to_field() == 16);
    let b_true = 2 == 2;
    assert(b_true.to_field() == 14);
    let b_false = 2 == 3;
    assert(b_false.to_field() == 3);
    let f = 13 as field;
    assert(f.to_field() == 13);
    let k_true = (12 as u32, true);
    assert(k_true.to_field() == 12);
    let k_false = (11 as u32, false);
    assert(k_false.to_field() == 32);
    let m = "String";
    assert(m.to_field() == 6);
    let unit = ();
    assert(unit.to_field() == 0);
    let point: Point2DAlias = [2, 3];
    assert(point.to_field() == 5);
    let i: field = 2;
    let j: field = 6;
    assert(f"i: {i}, j: {j}".to_field() == 52);
    assert(some_func.to_field() == 17);

    let mut y = 0 as u64;
    assert(y.to_field() == 66);
}
