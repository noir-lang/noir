// Regression test for https://github.com/noir-lang/noir/issues/12659
// Associated constant projections in struct-field type position should resolve.

pub trait HasSize {
    let N: u32;
}

pub struct PublicMutable<T> {}

impl<T> HasSize for PublicMutable<T> {
    let N: u32 = 1;
}

pub struct Wrapper<let N: u32> {
    pub value: [Field; N],
}

pub struct StorageArray {
    pub value: [Field; <PublicMutable<u128> as HasSize>::N],
}

pub struct StorageWrapper {
    pub value: Wrapper<<PublicMutable<u128> as HasSize>::N>,
}

fn main() {
    let _: StorageArray = StorageArray { value: [0] };
    let _: StorageWrapper = StorageWrapper { value: Wrapper { value: [0] } };
}
