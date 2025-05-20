use crate::{
    abis::side_effect::{Ordered, scoped::Scoped},
    address::AztecAddress,
    traits::{Deserialize, Empty, Serialize},
    utils::{arrays::array_concat, reader::Reader},
};

pub struct Counted<T> {
    pub inner: T,
    pub counter: u32,
}

impl<T> Counted<T> {
    pub fn new(inner: T, counter: u32) -> Self {
        Self { inner, counter }
    }
}

impl<T> Eq for Counted<T>
where
    T: Eq,
{
    fn eq(self, other: Self) -> bool {
        (self.inner == other.inner) & (self.counter == other.counter)
    }
}

impl<T> Empty for Counted<T>
where
    T: Empty,
{
    fn empty() -> Self {
        Self { inner: T::empty(), counter: 0 }
    }
}

impl<T, let N: u32> Serialize<N> for Counted<T>
where
    T: Serialize<N - 1>,
{
    fn serialize(self) -> [Field; N] {
        array_concat(self.inner.serialize(), [self.counter as Field])
    }
}

impl<T, let N: u32> Deserialize<N> for Counted<T>
where
    T: Deserialize<N - 1>,
{
    fn deserialize(fields: [Field; N]) -> Self {
        let mut reader = Reader::new(fields);
        let deserialized =
            Self { inner: reader.read_struct(T::deserialize), counter: reader.read_u32() };
        reader.finish();
        deserialized
    }
}

impl<T> Ordered for Counted<T> {
    fn counter(self) -> u32 {
        self.counter
    }
}

impl<T> Counted<T> {
    pub fn scope(self, contract_address: AztecAddress) -> Scoped<Counted<T>> {
        Scoped::new(self, contract_address)
    }
}

#[test]
fn serialization_of_empty_counted_field() {
    let item: Counted<Field> = Counted::empty();
    let serialized = item.serialize();
    let deserialized = Counted::deserialize(serialized);
    assert(item.eq(deserialized));
}

#[test]
fn serialization_of_non_empty_counted_field() {
    let item: Counted<Field> = Counted::new(12, 3);
    let serialized = item.serialize();
    let deserialized = Counted::deserialize(serialized);
    assert(item.eq(deserialized));
}
