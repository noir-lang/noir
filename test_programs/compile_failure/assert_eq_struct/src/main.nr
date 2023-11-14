struct myStruct {}

// `assert_eq` should not allow asserting equality between types for which `==` is not defined.
fn main(x : myStruct, y : pub myStruct) {
    assert_eq(x, y);
}
