trait Serialize {
    let N: u32;

    fn serialize(self) -> [Field; N];
}

// Testing using primitive types

impl Serialize for Field {
    let N: u32 = 1;

    fn serialize(self) -> [Field; Self::N] {
        [self]
    }
}

pub struct Foo {
    inner: Field,
}

impl Serialize for Foo {
    let N: u32 = <Field as Serialize>::N;

    fn serialize(self) -> [Field; Self::N] {
        [0; <Field as Serialize>::N]
    }
}

// Testing using no primitive types.

pub struct Bar {}

pub struct Baz {
    inner: Bar,
}

impl Serialize for Bar {
    let N: u32 = 1;

    fn serialize(self) -> [Field; Self::N] {
        [1]
    }
}

impl Serialize for Baz {
    let N: u32 = <Bar as Serialize>::N;

    fn serialize(self) -> [Field; Self::N] {
        [0; <Bar as Serialize>::N]
    }
}

fn main() {}
