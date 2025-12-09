use meta::{derive_deserialize, derive_serialize};

mod meta;

#[derive_via(derive_serialize)]
trait Serialize {
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

impl<let M: u32> Serialize for [Field; M] {
    let N: u32 = M;

    #[inline_always]
    fn serialize(self) -> [Field; Self::N] {
        self
    }
}

#[derive_via(derive_deserialize)]
trait Deserialize {
    let N: u32;

    fn deserialize(fields: [Field; N]) -> Self;
}

impl Deserialize for Field {
    let N: u32 = 1;

    #[inline_always]
    fn deserialize(fields: [Field; Self::N]) -> Self {
        fields[0]
    }
}

impl<let M: u32> Deserialize for [Field; M] {
    let N: u32 = M;

    #[inline_always]
    fn deserialize(fields: [Field; Self::N]) -> Self {
        fields
    }
}

#[derive(Serialize, Deserialize, Eq)]
struct Foo {
    x: Field,
    y: [Field; 3],
}

fn main() {
    let foo: Foo = Foo { x: 0, y: [1, 2, 3] };
    let serialized = foo.serialize();
    let deserialized: Foo = Deserialize::deserialize(serialized);
    assert_eq(foo, deserialized);
}
