fn main(input: Field, enable: bool) {
    if enable {
        let hash = no_predicate_function(input);
        // `EnableSideEffects` instruction from above instruction leaks out and removes the predicate from this call,
        // resulting in execution failure.
        unsafe { fail(hash) };
    }
}

#[no_predicates]
fn no_predicate_function(enable: Field) -> Field {
    // if-statement ensures that an `EnableSideEffects` instruction is emitted.
    if enable == 0 {
        1
    } else {
        0
    }
}

unconstrained fn fail(_: Field) {
    assert(false);
}
