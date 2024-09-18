struct EnumEmulation {
    a: Option<Field>,
    b: Option<Field>,
    c: Option<Field>,
}

unconstrained fn main() -> pub Field {
    let mut emulated_enum = EnumEmulation { a: Option::some(1), b: Option::none(), c: Option::none() };

    for _ in 0..1 {
        if emulated_enum.c.is_none() {
            emulated_enum.b = Option::some(10);
        }
        assert_eq(emulated_enum.a.unwrap(), 1);
    }

    emulated_enum.a = Option::some(2);
    assert_eq(emulated_enum.b.unwrap(), 10);
    emulated_enum.a.unwrap()
}
