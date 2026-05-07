// Regression: LICM must not replace induction-variable-dependent expressions
// that do not reference the induction variable.
unconstrained fn main(flag: bool) {
    let mut idx: u32 = 0;
    while (!flag) {
        if idx == 1 {
            break;
        }
        idx += 1;
    }
    assert(idx == 1);
}
