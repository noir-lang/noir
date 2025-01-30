// Tests a very simple program.
//
// The features being tested is using assert on brillig that is triggered through nested ACIR calls.
// We want to make sure we get a call stack from the original call in main to the failed assert.
fn main(x: Field) {
    assert(1 == fold_conditional_wrapper(!x as bool));
    assert(1 == fold_conditional_wrapper(x as bool));
}

#[fold]
fn fold_conditional_wrapper(x: bool) -> Field {
    fold_conditional(x)
}

#[fold]
fn fold_conditional(x: bool) -> Field {
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        conditional_wrapper(x)
    }
}

unconstrained fn conditional_wrapper(x: bool) -> Field {
    conditional(x)
}

unconstrained fn conditional(x: bool) -> Field {
    assert(x);
    1
}
