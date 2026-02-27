fn main() -> pub Field {
    // Safety: testing context
    let result = unsafe { foo() };
    assert(result == 1);
    result
}

unconstrained fn foo() -> Field {
    bar()
}

unconstrained fn bar() -> Field {
    while true {}
    1
}
