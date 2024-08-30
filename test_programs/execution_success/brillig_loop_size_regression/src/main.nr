struct EnumEmulation {
    a: Option<Field>,
    b: Option<Field>,
    c: Option<Field>,
}

unconstrained fn main() -> pub Field {
    let mut emulated_enum = EnumEmulation { a: Option::some(1), b: Option::none(), c: Option::none() };

    // Do this to optimize out loads in the loop
    // let copy_enum = emulated_enum;
    for _ in 0..1 {
        assert_eq(emulated_enum.a.unwrap(), 1);
    }

    // let foo = &mut 0;
    // let alias = foo;

    // let foo_load1 = *foo;
    // *alias = 7;
    // *foo = foo_load1; // This looks like it'd be optimized out

    // assert_eq(*foo, 7);

    emulated_enum.a = Option::some(2);
    emulated_enum.a.unwrap()
}
