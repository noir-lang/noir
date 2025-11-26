unconstrained fn foo(x: &mut u32) {
    assert_eq(*x, 1);
}

unconstrained fn bar(x: &mut u32, f: unconstrained fn(&mut u32)) {
    f(x);
}

unconstrained fn main() {
    let x = &mut 1_u32;
    bar(x, foo);
    bar(x, |x| foo(x));

    // Still panicking:
    // let f = |x| foo(x);
    // bar(x, f);
}
