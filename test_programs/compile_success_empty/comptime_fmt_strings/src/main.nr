fn main() {
    // format strings are lowered as normal strings
    let (s1, s2): (str<39>, str<4>) = comptime {
        let x: u32 = 4;
        let y: u32 = 5;

        // Can't print these at compile-time here since printing to stdout while
        // compiling breaks the test runner.
        let s1 = f"x is {x}, fake interpolation: {{y}}, y is {y}".as_quoted_str!();
        let s2 = std::mem::zeroed::<fmtstr<4, ()>>().as_quoted_str!();
        (s1, s2)
    };
    assert_eq(s1, "x is 4, fake interpolation: {y}, y is 5");
    assert_eq(s2, "\0\0\0\0");

    // Mainly test fmtstr::quoted_contents
    call!(glue(quote { hello }, quote { world }));

    // docs:start:as_quoted_str_test
    comptime {
        let x = 1;
        let f: str<_> = f"x = {x}".as_quoted_str!();
        assert_eq(f, "x = 0x01");
    }
    // docs:end:as_quoted_str_test
}

comptime fn glue(x: Quoted, y: Quoted) -> Quoted {
    f"{x}_{y}".quoted_contents()
}

fn hello_world() {}

comptime fn call(x: Quoted) -> Quoted {
    quote { $x() }
}
