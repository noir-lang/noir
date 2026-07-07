use std::mem::zeroed;

trait Trait {
    let N: u32;
}

impl Trait for Field {
    let N: u32 = 10;
}

fn main() {
    let something: [Field; <Field as Trait>::N] = zeroed();
    assert_eq(something.len(), <Field as Trait>::N);
}
