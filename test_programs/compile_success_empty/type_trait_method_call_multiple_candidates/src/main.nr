use dep::std::convert::From;

struct MyU128 {
    lo: Field,
    hi: Field,
}

impl MyU128 {
    pub fn from_u64s_le(lo: u64, hi: u64) -> MyU128 {
        MyU128 { lo: lo as Field, hi: hi as Field }
    }
}

impl From<u64> for MyU128 {
    fn from(value: u64) -> MyU128 {
        MyU128::from_u64s_le(value, 0)
    }
}

impl From<u32> for MyU128 {
    fn from(value: u32) -> MyU128 {
        MyU128::from(value as u64)
    }
}

fn main() {
    let x: u64 = 0;
    let mut small_int = MyU128::from(x);
    assert(small_int.lo == x as Field);
    let u32_3: u32 = 3;
    assert(MyU128::from(u32_3).lo == 3);
}
