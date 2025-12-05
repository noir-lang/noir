fn main() {
    // The initial assignment is okay.
    let mut foo: unconstrained fn() = foo_wrapper;

    // But then we point right at the oracle.
    // We could have pointed at it from the beginning, the front end would not have rejected it,
    // because the local variable is mutable and doesn't have a stable definition.
    foo = foo_oracle;

    // safety: Unsafe because are calling an oracle directly from constrained.
    unsafe {
        foo();
    }
}

#[oracle(foo)]
unconstrained fn foo_oracle() {}

unconstrained fn foo_wrapper() {
    foo_oracle();
}
