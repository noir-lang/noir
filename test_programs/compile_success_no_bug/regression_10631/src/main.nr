unconstrained fn foo(x: &mut u32) {
    assert_eq(*x, 1);
}

unconstrained fn bar(x: &mut u32, f: unconstrained fn(&mut u32)) {
    // If we generated a constrained lambda for `f`, then we would get a error in SSA validation
    // when we try to pass a mutable reference from the unconstrained `bar` to the constrained `f`.
    f(x);
}

unconstrained fn main() {
    let x = &mut 1_u32;

    // Passing the known unconstrained `foo` does not end up in a constrained lambda.
    bar(x, foo);

    // Passing a lambda, we can figure out that `bar` needs it to be unconstrained,
    // and avoid generating the constrained lambda.
    bar(x, |x| foo(x));

    // Here we don't know whether `f` will be used as constrained or unconstrained, or both.
    let f = |x| foo(x);

    // When passing `f` to `bar`, we know it's only interested in the unconstrained version.
    bar(x, f);
}
