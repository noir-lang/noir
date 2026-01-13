// If we generated a constrained lambda that calls `f`, then we would get a error in SSA validation,
// because it would be attempting to pass a mutable reference from a constrained to an unconstrained function.
unconstrained fn foo(_x: &mut u32) {}

// `bar` states that it wants an `unconstrained` lambda, which we can use for inference.
unconstrained fn bar(_f: unconstrained fn(&mut u32)) {}

// `baz` doesn't state that it wants an `unconstrained` lambda, however because it is itself
// unconstrained, we know it will only ever call functions monomorphized to be unconstrained.
unconstrained fn baz(_f: fn(&mut u32)) {}

fn main() {
    // safety: test
    unsafe {
        unconstrained_to_unconstrained();
    }
    constrained_to_unconstrained();
}

// // Test cases of creating lambdas in an unconstrained environment and passing them to unconstrained functions.
unconstrained fn unconstrained_to_unconstrained() {
    // Passing the known unconstrained `foo` does not end up in a constrained lambda.
    bar(foo);

    // The frontend understands that even though we need to coerce an unconstrained function
    // to match the type of a seemingly constrained lambda, it is okay because `baz` is unconstrained.
    baz(foo);

    // Passing an inline lambda, we can figure out that `bar` needs it to be unconstrained,
    // and avoid generating the constrained lambda.
    bar(|x| foo(x));

    // Passing an inline lambda, we can figure out that `baz` is unconstrained,
    // and avoid generating the constrained lambda.
    baz(|x| foo(x));

    // Here we don't know whether `f` will be used as constrained or unconstrained, or both;
    // it would normally be monomorphized as constrained and unconstrained, and cause an error even if unused.
    // But, since `unconstrained_to_unconstrained` is itself `unconstrained`, in this instance we know we
    // won't ever use the constrained variant, and can avoid creating it.
    let f = |x| foo(x);

    // When passing a variable lambda to `bar`, we know it's only interested in the unconstrained version,
    // but it doesn't matter, `f` has already been created.
    bar(f);
    baz(f);
}

// Test cases of creating lambdas in a constrained environment and passing them to unconstrained functions.
fn constrained_to_unconstrained() {
    // safety: test
    unsafe {
        // Passing the known unconstrained `foo` works when `unconstrained fn` is expected.
        bar(foo);

        // The frontend can also figure out that passing it to the unconstrained `baz` is okay.
        baz(foo);

        // `bar` states that it needs an unconstrained lambda, so the frontend can recognize that it would
        // not be passing a mutable reference from constrained to unconstrained.
        bar(|x| foo(x));

        // `baz` is unconstrained, so the frontend can recognize that it would
        // not be passing a mutable reference from constrained to unconstrained.
        baz(|x| foo(x));

        // Here we don't know whether `f` will be used as constrained or unconstrained, or both,
        // so the frontend rejects this because it looks like it would be passing a mutable reference
        // to an unconstrained function.
        // let f = |x| foo(x);
        // bar(f);
        // baz(f);
    }
}
