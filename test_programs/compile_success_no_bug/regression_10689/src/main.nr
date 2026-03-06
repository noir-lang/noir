use std::hash::{Hash, Hasher};

struct DummyHasher {}

impl Hasher for DummyHasher {
    fn finish(self) -> Field {
        0
    }

    fn write(&mut self, _input: Field) {}
}

fn main() {
    comptime {
        let mut dummy_hasher = DummyHasher {};
        // The zeroed value might be wrapped up in another data type, such as Option,
        // however, the Hash implementation of Option doesn't use the value unless it's Some,
        // so we never call the Hasher with the zeroed module, so this succeeds.
        Option::<Type>::none().hash(&mut dummy_hasher);
        Option::<Module>::none().hash(&mut dummy_hasher);
        Option::<TraitConstraint>::none().hash(&mut dummy_hasher);
        Option::<Quoted>::none().hash(&mut dummy_hasher);
        Option::<TraitDefinition>::none().hash(&mut dummy_hasher);
        Option::<TypeDefinition>::none().hash(&mut dummy_hasher);
        Option::<CtString>::none().hash(&mut dummy_hasher);
        Option::<FunctionDefinition>::none().hash(&mut dummy_hasher);
    }
}
