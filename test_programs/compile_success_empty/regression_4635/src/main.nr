trait FromField {
    fn from_field(field: Field) -> Self;
}

impl FromField for Field {
    fn from_field(value: Field) -> Self {
        value
    }
}

trait Deserialize<let N: u32> {
    fn deserialize(fields: [Field; N]) -> Self;
}

global AZTEC_ADDRESS_LENGTH: u32 = 1;

struct AztecAddress {
    inner: Field,
}

impl FromField for AztecAddress {
    fn from_field(value: Field) -> Self {
        Self { inner: value }
    }
}

impl Deserialize<AZTEC_ADDRESS_LENGTH> for AztecAddress {
    fn deserialize(fields: [Field; AZTEC_ADDRESS_LENGTH]) -> Self {
        AztecAddress::from_field(fields[0])
    }
}

impl Eq for AztecAddress {
    fn eq(self, other: Self) -> bool {
        self.inner == other.inner
    }
}

// Custom code

struct MyStruct<T> {
    a: T,
}

impl<T> Deserialize<1> for MyStruct<T>
where
    T: FromField,
{
    fn deserialize(fields: [Field; 1]) -> Self {
        Self { a: FromField::from_field(fields[0]) }
    }
}

fn main() {
    let fields = [5; 1];
    let foo = MyStruct::deserialize(fields); // Note I don't specify T here (the type of `foo.a`)
    let bar = AztecAddress { inner: 5 };

    // Here `T` is apparently inferred to be `AztecAddress`, presumably because of the comparison.
    assert(foo.a == bar);
}
