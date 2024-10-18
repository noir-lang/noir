trait Fieldable {
    fn to_field(self) -> Field;
}

impl Fieldable for u32 {
    fn to_field(self) -> Field {
        let res = self as Field;
        res * 3
    }
}

impl Fieldable for [u32; 3] {
    fn to_field(self) -> Field {
        let res = self[0] + self[1] + self[2];
        res as Field
    }
}

impl Fieldable for bool {
    fn to_field(self) -> Field {
        if self {
            14
        } else {
            3
        }
    }
}

impl Fieldable for (u32, bool) {
    fn to_field(self) -> Field {
        if self.1 {
            self.0 as Field
        } else {
            32
        }
    }
}

impl Fieldable for Field {
    fn to_field(self) -> Field {
        self
    }
}

impl Fieldable for str<6> {
    fn to_field(self) -> Field {
        6
    }
}

impl Fieldable for () {
    fn to_field(self) -> Field {
        0
    }
}

type Point2D = [Field; 2];
type Point2DAlias = Point2D;

impl Fieldable for Point2DAlias {
    fn to_field(self) -> Field {
        self[0] + self[1]
    }
}

impl Fieldable for fmtstr<14, (Field, Field)> {
    fn to_field(self) -> Field {
        52
    }
}

impl Fieldable for fn(u32) -> u32 {
    fn to_field(self) -> Field {
        self(10) as Field
    }
}

fn some_func(x: u32) -> u32 {
    x * 2 - 3
}

impl Fieldable for u64 {
    fn to_field(self) -> Field {
        66 as Field
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
    let f = 13 as Field;
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
    let i: Field = 2;
    let j: Field = 6;
    assert(f"i: {i}, j: {j}".to_field() == 52);
    assert(some_func.to_field() == 17);

    let mut y = 0 as u64;
    assert(y.to_field() == 66);
}
