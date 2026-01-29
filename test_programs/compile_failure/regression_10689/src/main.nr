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
        // These are variants of QuotedType available only at comptime.
        // The comptime `zeroed` builtin creates a `Value::Zeroed(<type>)` for them,
        // but doesn't expect that this will be used, so get an error from the interpreter.
        std::mem::zeroed::<Type>().hash(&mut dummy_hasher);
        std::mem::zeroed::<Module>().hash(&mut dummy_hasher);
        std::mem::zeroed::<TraitConstraint>().hash(&mut dummy_hasher);
        std::mem::zeroed::<Quoted>().hash(&mut dummy_hasher);
        std::mem::zeroed::<TraitDefinition>().hash(&mut dummy_hasher);
        std::mem::zeroed::<TypeDefinition>().hash(&mut dummy_hasher);
        std::mem::zeroed::<CtString>().hash(&mut dummy_hasher);
        std::mem::zeroed::<FunctionDefinition>().hash(&mut dummy_hasher);
    }
}
