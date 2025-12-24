pub trait Serialize {
    let N: u32;

    fn serialize(self);
}

impl Serialize for Field {
    let N: u32 = 1;

    fn serialize(self) {}
}

impl<T, let M: u32> Serialize for [T; M]
where
    T: Serialize,
{
    let N: u32 = <T as Serialize>::N * M;

    fn serialize(self) {
        std::println(Self::N);
        std::println(<T as Serialize>::N);
        std::println(M);
        assert_eq(<T as Serialize>::N * M, Self::N);
    }
}

fn main() {
    let nested_array = [[1], [2]];

    let _ = nested_array.serialize();
}
