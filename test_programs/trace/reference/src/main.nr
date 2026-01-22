struct TestStruct {
    a: u32,
    b: u32,
}

fn testfunc(s: &mut TestStruct) {
}

fn main(x: Field, y: pub Field) {
    assert(x != y);
    let mut q = TestStruct { a: 5, b: 10 };
    testfunc(&mut q);
}
