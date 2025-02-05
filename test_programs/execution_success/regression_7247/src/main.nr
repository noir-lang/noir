fn main(
    a: u32,
    b: u32,
    c: u32,
) {
    /// Safety: Test
    let vb = unsafe { via_brillig(a) };
    let va = via_acir(a);
    assert_eq(vb, b);
    assert_eq(va, b);

    /// Safety: Test
    let bat = unsafe { brillig_apply_twice(a, times_ten) };
    assert_eq(bat, b, "should be a*100");

    let mat = mixed_apply_thrice(a, times_ten);
    assert_eq(mat, c, "should be a*1000");
}

// The lambdas created within should be constrained.
fn via_acir(a: u32) -> u32 {
    times100(a)
}

// The lambdas created within this should be unconstrained,
unconstrained fn via_brillig(a: u32) -> u32 {
    times100(a)
}

// Example of a higher order function that will be transformed
// to call lambdas via `apply` functions. Those should only call
// one version of the `|x| x * 10` lambda, not both `acir` and `brillig`.
fn apply_twice(a: u32, f: fn(u32) -> u32) -> u32 {
    f(f(a))
}

// Example of an unconstrained higher order function to which we pass
// a lambda defined in constrained code; we want it to call a `brillig` lambda.
unconstrained fn brillig_apply_twice(a: u32, f: fn(u32) -> u32) -> u32 {
    f(f(a))
}

// Example of a constrained function that gets a constrained lambda,
// which it passes on to unconstrained code and also calls it locally,
// so the lambda is gets is called from both kinds of environments.
fn mixed_apply_thrice(a: u32, f: fn(u32) -> u32) -> u32 {
    /// Safety: Test
    let aa = unsafe { brillig_apply_twice(a, f) };
    let aaa = f(aa);
    aaa
}

// Example of creating a lambda and passing it on. Whether it's constrained or unconstrained
// depends on whether we call it from `via_acir` or `via_brillig`.
fn times100(a: u32) -> u32 {
    apply_twice(a, |x| x * 10)
}

// Example of a constrained function we pass to both constrained and unconstrained by name.
fn times_ten(x: u32) -> u32 {
    // Arbitrary `if` statement to trigger `EnableSideEffect` during flattening.
    if x > 100000 {
        x * 10 - 1
    } else {
        x * 10
    }
}
