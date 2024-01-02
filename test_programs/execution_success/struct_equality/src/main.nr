// This test demonstrates how SSA optimizations breaks down a struct equality assertion
// to assert directly on the struct's fields rather than using a predicate.

use dep::std::ops::Eq;

struct MyStruct {
    foo: Field,
    bar: bool,
    baz: [u8; 3]
}

impl Eq for MyStruct {
    fn eq(self, other: MyStruct) -> bool {
        (self.foo == other.foo) & (self.bar == other.bar) & (self.baz == other.baz)
    }
}

fn main(x: MyStruct) {
    let y = MyStruct { foo: 42, bar: true, baz: [1, 2, 3] };
    assert(x.eq(y));
}
