#[inline_never]
unconstrained fn returns_true() -> bool {
    true
}

unconstrained fn f(mut a: bool, mut b: bool) -> bool {
    let r: &mut bool = if returns_true() { &mut a } else { &mut b };
    a = true;
    *r
}

unconstrained fn main() {
    let expected = comptime { f(false, false) };
    let result = f(false, false);
    assert(result == expected);
}
