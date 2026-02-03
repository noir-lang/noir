fn main() {
    // The initial assignment is okay.
    let mut foo: unconstrained fn() = foo_wrapper;

    // But then we point right at the oracle.
    // We could have pointed at it from the beginning, the front end would not have rejected it,
    // because the local variable is mutable and doesn't have a stable definition.
    foo = foo_oracle;

    // The lints also don't apply to data structures.
    let foo_tuple = (foo_oracle, foo_oracle);

    // And since we don't lint data structures, we can allow the non-mutable case as well.
    let foo_immutable = foo_oracle;

    // safety: Unsafe because are calling an oracle directly from constrained,
    //         but the proxies pass will create two unconstrained proxies,
    //         through which we can call the oracle after all.
    unsafe {
        foo();
        (foo_tuple.0)();
        foo_immutable();
    }
}

#[oracle(foo)]
unconstrained fn foo_oracle() {}

unconstrained fn foo_wrapper() {
    foo_oracle();
}
