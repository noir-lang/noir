// Regression test for noir-lang/noir-claude#1303: LICM used to rewrite the
// checked `i - 1` into `unchecked_sub` for this decreasing `while` loop,
// returning the wrapped value 4294967295 instead of trapping on underflow.
unconstrained fn main() -> pub u32 {
    let mut i: u32 = 5;
    while i < 10 {
        i = i - 1;
    }
    i
}
