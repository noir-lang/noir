fn main() {
    let c = comptime { func_1((0, true)) };
    // Safety: no
    let b = unsafe { func_1((0, true)) };
    assert(c);
    assert(b);
}

unconstrained fn func_1(a: (Field, bool)) -> bool {
    let mut b = a;
    b.1 = false;
    a.1
}
