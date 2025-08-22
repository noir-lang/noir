fn main() {
    // Safety: testing context
    unsafe {
        foo();
    }
}

unconstrained fn foo() {
    bar();
}

unconstrained fn bar() {
    bar();
}
