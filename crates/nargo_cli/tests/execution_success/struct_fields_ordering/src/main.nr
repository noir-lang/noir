// Note that fields are not in alphabetical order.
// We want to check that this ordering is maintained
struct myStruct {
    foo: u32,
    bar: Field,
}

fn main(y : pub myStruct) {
    assert(y.foo == 5);
    assert(y.bar == 7);
}

