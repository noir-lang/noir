struct myStruct {
    foo: u32,
    bar: Field,
}

fn main(x : Field, y : pub myStruct) {
    // constrain x != y.foo as Field; // Will fail to prove if uncommented
    constrain x != y.bar;
}