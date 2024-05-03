fn main(x: u32) {
    // We want to call a foldable function after a call to a function that is set to be inlined
    assert(increment(x) == x + 1);
    foo(x);
}

#[fold]
fn foo(x: u32) {
    assert(x == 1);
}

fn increment(x: u32) -> u32 {
    x + 1
}
