use dep::std;

// Note that fields are not in alphabetical order.
// We want to check that this ordering is maintained
struct myStruct {
    foo: u32,
    bar: Field,
}

fn main(y : pub myStruct) {
    std::assert_eq(y.foo, 5);
    std::assert_eq(y.bar, 7);
}

