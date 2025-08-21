use std::static_assert;

fn main(x: Field) {
    let string = f"Assertion failed: {x}";
    static_assert(true, string);
}
