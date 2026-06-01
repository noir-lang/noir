// Regression test: constant folding's CSE must not hoist instructions
// into loop headers when it can't escape to the pre-header.
// Here `s * s` appears in both the loop body and after the loop;
// hoisting it into the header would break loop unrolling, which only
// maps header parameters (not instruction results) to final-iteration values.
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
