comptime fn my_macro(x: i32, y: i32) -> Quoted {
    // Current version of macros in Noir are not hygienic
    // so we can quote a and b here and expect them to resolve
    // to the a and b in main at the callsite of my_macro.
    quote {
        $x + $y + a + b
    }
}

fn main() {
    let a = 100;
    let b = 200;
    let result = my_macro!(1, 2);
    assert_eq(result, 1 + 2 + a + b);
}
