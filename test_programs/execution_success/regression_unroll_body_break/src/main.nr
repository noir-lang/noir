// Regression: a for-loop with a conditional break from a body block (not the header)
// used to panic with "destination not in original loop" during Brillig unrolling.
// The `is_fully_executed` check only examined the header's exit, missing exits from
// body blocks, which allowed the unroller to encounter blocks outside the loop.
unconstrained fn main() {
    let mut sum = 0_u32;
    for i in 0..3 {
        if should_break(i) {
            break;
        }
        sum += i;
    }
    // i=0: no break, sum=0; i=1: no break, sum=1; i=2: break
    assert_eq(sum, 1);
}

unconstrained fn should_break(i: u32) -> bool {
    i == 2
}
