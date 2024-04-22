// Tests a very simple program.
// 
// The features being tested is using assert on brillig that is triggered through nested ACIR calls.
// We want to make sure we get a call stack from the original call in main to the failed assert.
fn main(x: field) {
    assert(1 == fold_conditional_wrapper(x as bool));
}

#[fold]
fn fold_conditional_wrapper(x: bool) -> field {
    fold_conditional(x)
}

#[fold]
fn fold_conditional(x: bool) -> field {
    conditional_wrapper(x)
}

unconstrained fn conditional_wrapper(x: bool) -> field {
    conditional(x)
}

unconstrained fn conditional(x: bool) -> field {
    assert(x);
    1
}
