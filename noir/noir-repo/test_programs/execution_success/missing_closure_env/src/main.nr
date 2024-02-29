fn main(x: Field) {
    let x1 = &mut 42;
    let set_x1 = |y| { *x1 = y; };

    assert(*x1 == 42);
    set_x1(44);
    assert(*x1 == 44);
    set_x1(*x1);
    assert(*x1 == 44);
    assert(x == 42);
}

#[test]
fn test_main() {
    main(42);
}
