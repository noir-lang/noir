#[some_attribute]
#[another_attribute]
struct SomeStruct {
    a: Field,
    b: Field
}

fn main() {}

// Check that add_attribute and has_named_attribute work well

#[add_attribute]
struct Foo {

}

comptime fn add_attribute(s: StructDefinition) {
    assert(!s.has_named_attribute(quote { foo }));
    s.add_attribute("foo");
    assert(s.has_named_attribute(quote { foo }));
}
