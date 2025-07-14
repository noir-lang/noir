pub trait Serialize<let N: u32> {
    fn serialize(self) -> [Field; N];
}

pub struct Counted<T> {
    pub inner: T,
}

pub fn append<T, let N: u32>(array1: [T; N]) -> [T; N + 1] {
    [array1[0]; N + 1]
}

impl<T, let N: u32> Serialize<N> for Counted<T>
where
    T: Serialize<N - 1>,
{
    fn serialize(self) -> [Field; N] {
        append(self.inner.serialize())
    }
}

fn main() {}
