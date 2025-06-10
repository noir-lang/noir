trait Serialize {
    let Size: u32;

    fn serialize(self) -> [Field; Self::Size];
}

impl<A, B> Serialize for (A, B)
where
    A: Serialize,
    B: Serialize,
{
    let Size: u32 = <A as Serialize>::Size + <B as Serialize>::Size;

    fn serialize(self: Self) -> [Field; Self::Size] {
        let _ = self.0.serialize();
        [0; Self::Size]
    }
}

impl<T, let N: u32> Serialize for [T; N]
where
    T: Serialize,
{
    let Size: u32 = <T as Serialize>::Size * N;

    fn serialize(self: Self) -> [Field; Self::Size] {
        [0; Self::Size]
    }
}

impl Serialize for Field {
    let Size: u32 = 1;

    fn serialize(self) -> [Field; Self::Size] {
        [self]
    }
}

fn main() {
    let x = (((1, [2, 3, 4]), [5, 6, 7, 8]), 9);
    assert_eq(x.serialize().len(), 9);
}
