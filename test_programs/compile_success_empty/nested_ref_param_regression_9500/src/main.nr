fn main() {
    let ref = &mut 0;
    let ref3 = &mut &mut 0;
    foo(ref, ref, ref3);
    assert_eq(*ref, 2);
}

fn foo(ref1: &mut Field, ref2: &mut Field, _ref3: &mut &mut Field) {
    *ref1 = 1;
    *ref2 = 2;
    for _ in 0..10 {}
}
