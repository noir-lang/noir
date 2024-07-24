fn main() {
    // format strings are lowered as normal strings
    let (s1, s2): (str<39>, str<4>) = comptime {
        let x = 4;
        let y = 5;

        // Can't print these at compile-time here since printing to stdout while
        // compiling breaks the test runner.
        let s1 = f"x is {x}, fake interpolation: \{y}, y is {y}";
        let s2 = std::unsafe::zeroed::<fmtstr<4, ()>>();
        (s1, s2)
    };
    assert_eq(s1, "x is 4, fake interpolation: {y}, y is 5");
    assert_eq(s2, "\0\0\0\0");
}
