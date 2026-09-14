// A lambda given to a *constrained* method must stay constrained even when the method call sits
// inside the argument list of an unconstrained call. If it is inferred unconstrained instead, the
// constrained `map` dispatches into a Brillig lambda and
// `check_for_missing_brillig_constraints` reports it, which is what this directory checks for.
unconstrained fn batch_invert(_x: [Field; 2]) {}

fn main(x: Field) -> pub Field {
    let arr: [Field; 2] = [x, x];
    // Safety: the result is only read by unconstrained code
    unsafe {
        batch_invert(arr.map(|v: Field| v + 1));
    }
    x
}
