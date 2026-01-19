fn main(x: u32) {
    if (x == 43) {
        let _ = foo(x);
    }
    assert(foo(x) != 0);
}

fn foo(input: u32) -> u32 {
    assert(input != 0);
    input % 100
}
