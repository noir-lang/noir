// Regression test: loop unrolling must propagate instruction results
// defined in the loop header (not just block parameters) to exit blocks.
// When constant folding's CSE hoists `s * s` into the loop header
// (as common dominator of body and exit), unrolling must map that
// instruction result to its final-iteration value.
fn main(x: Field) {
    let result = compute(x);
    assert(result != 0);
}

#[no_predicates]
fn compute(x: Field) -> Field {
    let mut s = x;
    for _i in 0..1 {
        s = s * s + 1;
    }
    s * s
}
