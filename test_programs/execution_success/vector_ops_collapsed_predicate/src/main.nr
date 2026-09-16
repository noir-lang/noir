// Popping from a statically-empty slice inside a conditional collapses the branch
// predicate to a constant zero at the ACIR level (the empty-pop assertion pins it),
// which in turn folds the length of an unrelated, still-live slice to a constant zero.
// A subsequent `remove` on that slice must not turn the folded length into `0 - 1` in
// the field: `push_back` consumes the result as a `u32` length, and the compiler must
// not crash converting it. With `g0 = true` the branch is inactive and `main` returns 0.
fn main(g0: bool, d: bool, i: u32) -> pub Field {
    let v = @[1];
    let mut w = @[1];
    if d {
        w = w.push_back(5);
    }
    let mut out = 0;
    if !g0 {
        let (v2, _a) = v.remove(i);
        let (_v3, _b) = v2.pop_back();
        let (w2, _c) = w.pop_back();
        let (w3, _e) = w2.remove(i);
        let w4 = w3.push_back(9);
        out = w4[0];
    }
    out
}
