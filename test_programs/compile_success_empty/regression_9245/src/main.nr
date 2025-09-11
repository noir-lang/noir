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

impl<let M: u32> Deserialize for [Field; M] {
    let N: u32 = <Field as Deserialize>::N * M;

    fn deserialize(fields: [Field; Self::N]) -> Self {
        fields
    }
}

pub struct Log<let O: u32> {
    pub fields: [Field; O],
}

impl<let O: u32> Deserialize for Log<O> {
    let N: u32 = <[Field; O] as Deserialize>::N;

    fn deserialize(fields: [Field; Self::N]) -> Self {
        Self { fields: <[Field; O] as Deserialize>::deserialize(fields) }
    }
}

fn main() {
    let serialized = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let log = Log::<10>::deserialize(serialized);
    assert_eq(serialized, log.fields);
}
