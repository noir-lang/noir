pub struct Struct<T> {
    pub x: T,
}

pub trait From2<T> {
    fn from2(input: T) -> Self;
}

impl From2<u64> for Struct<u64> {
    fn from2(_: u64) -> Struct<u64> {
        Struct { x: 0 }
    }
}

impl From2<u32> for Struct<u32> {
    fn from2(_: u32) -> Struct<u32> {
        Struct { x: 0 }
    }
}

fn main() {
    let x = 1;
    let f = Struct::<u32>::from2;
    let _ = f(x);
}
