trait Serialize {
    let Size: u32;

    // Note that Rust disallows referencing constants here!
    fn serialize(self) -> [Field; Self::Size];
}

impl<A, B> Serialize for (A, B)
where
    A: Serialize,
    B: Serialize,
{
    let Size: u32 = <A as Serialize>::Size + <B as Serialize>::Size;

    fn serialize(self: Self) -> [Field; Self::Size] {
        let mut array: [Field; Self::Size] = std::mem::zeroed();
        let a = self.0.serialize();
        let b = self.1.serialize();

        for i in 0..a.len() {
            array[i] = a[i];
        }
        for i in 0..b.len() {
            array[i + a.len()] = b[i];
        }
        array
    }
}

impl<T, let N: u32> Serialize for [T; N]
where
    T: Serialize,
{
    let Size: u32 = <T as Serialize>::Size * N;

    fn serialize(self: Self) -> [Field; Self::Size] {
        let mut array: [Field; Self::Size] = std::mem::zeroed();
        let mut array_i = 0;

        for elem in self {
            let elem_fields = elem.serialize();

            for i in 0..elem_fields.len() {
                array[array_i] = elem_fields[i];
                array_i += 1;
            }
        }

        array
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
