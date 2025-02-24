#['some_attribute]
#['another_attribute]
pub struct SomeStruct {
    a: Field,
    b: Field,
}

fn main() {}

// Check that add_attribute and has_named_attribute work well

#[abi(something)]
#[add_attribute]
pub struct Foo {}

comptime fn add_attribute(s: StructDefinition) {
    assert(!s.has_named_attribute("foo"));
    s.add_attribute("foo");
    assert(s.has_named_attribute("foo"));

    assert(s.has_named_attribute("abi"));
}
