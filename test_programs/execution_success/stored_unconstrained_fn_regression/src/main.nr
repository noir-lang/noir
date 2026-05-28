use std::runtime::is_unconstrained;

struct Holder {
    f: unconstrained fn(u32) -> u32,
}

fn choose(x: u32) -> u32 {
    if is_unconstrained() { x + 100 } else { x + 1 }
}

fn call_holder(holder: Holder, x: u32) -> u32 {
    // Safety: testing stored unconstrained fn dispatch
    unsafe { (holder.f)(x) }
}

fn main() -> pub u32 {
    let f: unconstrained fn(u32) -> u32 = choose;
    // Safety: testing stored unconstrained fn dispatch
    let from_local = unsafe { f(7) };
    assert_eq(from_local, 107);

    let holder = Holder { f: choose };
    let from_field = call_holder(holder, 11);
    assert_eq(from_field, 111);

    from_local + from_field
}
