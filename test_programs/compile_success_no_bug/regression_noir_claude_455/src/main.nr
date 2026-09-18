// A lambda given to the constrained `map` is constrained, so an `assert` in its body becomes an
// ACIR assertion the verifier checks -- including when the method call sits inside the argument
// list of an unconstrained call. If the lambda is inferred unconstrained instead, the assertion
// only runs in Brillig, the constrained `map` dispatches into it, and
// `check_for_missing_brillig_constraints` reports it, which is what this directory checks for.
unconstrained fn hint(_x: [Field; 2]) {}

fn main(x: [Field; 2]) {
    // Safety: the hint's result is discarded
    unsafe {
        hint(x.map(|v: Field| {
            assert(v == 7, "v must be 7");
            v
        }));
    }
}
