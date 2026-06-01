use std::meta::ctstring::AsCtString;

#['some_attribute]
#['another_attribute]
pub struct SomeStruct {
    a: Field,
    b: Field,
}

fn main() {}

// Check that add_attribute and has_named_attribute work well

contract moo {
    #[super::add_attribute]
    pub struct Foo {}
}

comptime fn add_attribute(s: TypeDefinition) {
    assert(!s.has_named_attribute("abi"));
    s.add_abi("foo".as_ctstring());
    assert(s.has_named_attribute("abi"));
}
