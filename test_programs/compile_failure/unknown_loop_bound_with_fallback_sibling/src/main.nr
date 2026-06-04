// A nested loop whose inner bound depends on the outer induction variable is only
// unrollable via the ACIR InsideOut->OutsideIn fallback. Paired with an independent
// runtime-bound sibling loop, the fallback used to discard the sibling's genuine
// `UnknownLoopBound` error and leave a residual loop, panicking in a later pass.
// Compilation must instead report the loop bound error cleanly.
fn main(n: u32, x: Field) {
    for i in 0..2 {
        for j in 0..i {
            assert(j < 2);
        }
    }

    for k in 0..n {
        assert(x == x);
        assert(k < n);
    }
}
