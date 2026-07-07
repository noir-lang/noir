fn main() {
    let string = &mut "OOPS";
    let param = [string];
    assert_eq(*param[0], "OOPS");
    foo(param);
    assert_eq(*param[0], "GOOD");
}

fn foo(param: [&mut str<4>; 1]) {
    let s = param[0];
    *s = "GOOD";
    for _ in 0..1 {}
}
