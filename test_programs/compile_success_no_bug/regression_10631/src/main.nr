// If we generated a constrained lambda that calls `f`, then we would get a error in SSA validation,
// because it would be attempting to pass a mutable reference from a constrained to an unconstrained function.
unconstrained fn foo(x: &mut u32) {
    assert_eq(*x, 1);
}

// `bar` states that it wants an `unconstrained` lambda, which we can use for inference.
unconstrained fn bar(x: &mut u32, f: unconstrained fn(&mut u32)) {
    f(x);
}

// `baz` doesn't state that it wants an `unconstrained` lambda, however because it is itself
// unconstrained, we know it will only ever call functions monomorphized to be unconstrained.
unconstrained fn baz(x: &mut u32, f: fn(&mut u32)) {
    f(x);
}

unconstrained fn main() {
    let x = &mut 1_u32;

    // Passing the known unconstrained `foo` does not end up in a constrained lambda.
    bar(x, foo);

    // This is rejected by the frontend, as an unsafe conversion of an unconstrained lambda to constrained.
    // baz(x, foo);

    // Passing an inline lambda, we can figure out that `bar` needs it to be unconstrained,
    // and avoid generating the constrained lambda.
    bar(x, |x| foo(x));

    // Passing an inline lambda, we can figure out that `baz` is unconstrained,
    // and avoid generating the constrained lambda.
    baz(x, |x| foo(x));

    // Here we don't know whether `f` will be used as constrained or unconstrained, or both;
    // it will be monomorphized as constrained and unconstrained, and cause an error even if unused.
    // But since `main` is itself `unconstrained`, in this instance we know we won't ever use the constrained variant.
    let f = |x| foo(x);

    // When passing a variable lambda to `bar`, we know it's only interested in the unconstrained version,
    // but it doesn't matter, `f` has already been created.
    bar(x, f);
    baz(x, f);
}
