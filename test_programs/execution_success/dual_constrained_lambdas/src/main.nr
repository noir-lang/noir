fn main() {
    foo(|| std::runtime::is_unconstrained());
}

// #7289 originates from a lambda being used in both acir & brillig
fn foo(is_brillig: fn() -> bool) {
    // If this is true the `--force-brillig` flag was passed
    let force_brillig = std::runtime::is_unconstrained();

    assert(!acir_function(is_brillig) | force_brillig);

    // safety:
    unsafe {
        assert(brillig_function(is_brillig));
        assert(brillig_function2(is_brillig));
    };
}

fn acir_function(f: fn() -> bool) -> bool {
    f()
}

unconstrained fn brillig_function(f: fn() -> bool) -> bool {
    f()
}

// acir_function should become unconstrained when called in
// an unconstrained context - and this should apply to the lambda
// it takes as an argument as well.
unconstrained fn brillig_function2(f: fn() -> bool) -> bool {
    acir_function(f)
}
