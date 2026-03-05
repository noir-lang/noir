fn main() {
    // safety: please no warning
    let one = unsafe { foo([-2, -1]) };
    println(one);
    assert_eq(1, one);
}

unconstrained fn foo(a: [Field; 2]) -> Field {
    let _ = bar(a);
    bar(a)
}

unconstrained fn bar(mut b: [Field; 2]) -> Field {
    b[1] = 3 + b[1];
    let mut running = true;
    while running {
        b[0] = 3 + b[0];
        running = false;
    }
    b[0]
}
