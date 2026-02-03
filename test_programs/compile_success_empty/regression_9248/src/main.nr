pub trait Deserialize {
    let N: u32;

    fn deserialize(fields: [Field; N]) -> Self;
}

impl Deserialize for Field {
    let N: u32 = 1;

    fn deserialize(fields: [Field; Self::N]) -> Self {
        fields[0]
    }
}

impl<T, let M: u32> Deserialize for [T; M]
where
    T: Deserialize,
{
    let N: u32 = <T as Deserialize>::N * M;

    fn deserialize(fields: [Field; Self::N]) -> Self {
        let mut result: [T; M] = std::mem::zeroed();

        for i in 0..M {
            let mut element_fields = [0; <T as Deserialize>::N];
            for j in 0..<T as Deserialize>::N {
                element_fields[j] = fields[i * <T as Deserialize>::N + j];
            }
            result[i] = T::deserialize(element_fields);
        }

        result
    }
}

pub struct Log<let M: u32> {
    pub fields: [Field; M],
}

impl<let M: u32> Deserialize for Log<M> {
    let N: u32 = M;

    fn deserialize(serialized: [Field; Self::N]) -> Self {
        let fields = <[Field; M] as Deserialize>::deserialize(serialized);
        Self { fields }
    }
}

fn main() {}
