
pub trait Serialize {
    let N: u32;

    fn serialize(self) -> [Field; N];
}

impl Serialize for Field {
    let N: u32 = 1;

    #[inline_always]
    fn serialize(self) -> [Field; Self::N] {
        [self]
    }
}


impl<T, let M: u32> Serialize for [T; M]
where
    T: Serialize,
{
    let N: u32 = <T as Serialize>::N * M;

    #[inline_always]
    fn serialize(self) -> [Field; Self::N] {
        let mut result: [Field; _] = std::mem::zeroed();
        std::println(Self::N);
        std::println(<T as Serialize>::N);
        std::println(M);
        for i in 0..M {
            let serialized_t = self[i].serialize();
            for j in 0..<T as Serialize>::N {
                result[i * <T as Serialize>::N + j] = serialized_t[j];
            }
        }
        result
    }
}

fn main() {
    let nested_array = [[1], [2]];

    let _ = nested_array.serialize();
}