unconstrained fn foo(cond: bool) -> bool {
    let mut b = false;
    let c = if cond { &mut b } else { &mut false };
    b = !b;
    b = *c;
    b
}

fn main() -> pub bool {
    let expected = comptime {
        unsafe {
            foo(true)
        }
    };
    let result = unsafe { foo(true) };
    assert(result == expected);
    unsafe {
        foo(true)
    }
}
