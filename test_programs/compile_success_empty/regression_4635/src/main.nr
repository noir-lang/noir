trait Fromfield {
    fn from_field(value: field) -> Self;
}

impl Fromfield for field {
    fn from_field(value: field) -> Self {
        value
    }
}

trait Deserialize<N> {
    fn deserialize(fields: [field; N]) -> Self;
}

global AZTEC_ADDRESS_LENGTH = 1;

struct AztecAddress {
    inner : field
}

impl Fromfield for AztecAddress {
    fn from_field(value: field) -> Self {
        Self { inner: value }
    }
}

impl Deserialize<AZTEC_ADDRESS_LENGTH> for AztecAddress {
    fn deserialize(fields: [field; AZTEC_ADDRESS_LENGTH]) -> Self {
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
    a: T
}

impl<T> Deserialize<1> for MyStruct<T> {
    fn deserialize(fields: [field; 1]) -> Self where T: Fromfield {
        Self{ a: Fromfield::from_field(fields[0]) }
    }
}

fn main() {
    let fields = [5; 1];
    let foo = MyStruct::deserialize(fields); // Note I don't specify T here (the type of `foo.a`)

    let bar = AztecAddress { inner: 5 };

    // Here `T` is apparently inferred to be `AztecAddress`, presumably because of the comparison.
    assert(foo.a == bar);
}
