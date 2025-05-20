use crate::{
    abis::side_effect::{counted::Counted, Ordered},
    address::AztecAddress,
    tests::types::TestValue,
    traits::{Deserialize, Empty, Serialize, ToField},
    utils::{arrays::array_concat, reader::Reader},
};

pub struct Scoped<T> {
    pub inner: T,
    pub contract_address: AztecAddress,
}

impl<T> Scoped<T> {
    pub fn new(inner: T, contract_address: AztecAddress) -> Self {
        Self { inner, contract_address }
    }
}

impl<T> Eq for Scoped<T>
where
    T: Eq,
{
    fn eq(self, other: Self) -> bool {
        (self.inner == other.inner) & (self.contract_address == other.contract_address)
    }
}

impl<T> Empty for Scoped<T>
where
    T: Empty,
{
    fn empty() -> Self {
        Self { inner: T::empty(), contract_address: AztecAddress::empty() }
    }
}

impl<T, let N: u32> Serialize<N> for Scoped<T>
where
    T: Serialize<N - 1>,
{
    fn serialize(self) -> [Field; N] {
        array_concat(self.inner.serialize(), [self.contract_address.to_field()])
    }
}

impl<T, let N: u32> Deserialize<N> for Scoped<T>
where
    T: Deserialize<N - 1>,
{
    fn deserialize(fields: [Field; N]) -> Self {
        let mut reader = Reader::new(fields);
        let deserialized = Self {
            inner: reader.read_struct(T::deserialize),
            contract_address: reader.read_struct(AztecAddress::deserialize),
        };
        reader.finish();
        deserialized
    }
}

impl<T> Ordered for Scoped<Counted<T>> {
    fn counter(self) -> u32 {
        self.inner.counter()
    }
}

#[test]
fn serialization_of_empty_scoped() {
    let item: Scoped<TestValue> = Scoped::empty();
    let serialized = item.serialize();
    let deserialized: Scoped<TestValue> = Scoped::deserialize(serialized);
    assert(item.eq(deserialized));
}
