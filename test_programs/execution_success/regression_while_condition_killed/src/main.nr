// A `while` condition's read of an array must clone even when the loop body unconditionally
// reassigns that array.
//
// The loop's exit edge leaves from the condition, so the final condition evaluation is followed
// by no reassignment at all: the buffer it consumed is the one still live after the loop. Moving
// that read lets `f` mutate `x`'s own storage through its by-value parameter.
//
// `f([1, 2, 3], 0)` returns `2 == 7`, i.e. false, so the body never runs and `x` is still
// `[1, 2, 3]`. A missing clone makes `x[0]` read the `99` that `f` wrote (or fails SSA validation
// in a debug build). The index must come from a witness: a constant index is rewritten into a
// fresh array by a later SSA pass, which hides the wrong value without restoring the clone.
unconstrained fn main(i: u32) {
    let mut x = [1, 2, 3];
    while f(x, i) {
        x = [7, 7, 7];
    }
    assert_eq(x[0], 1);
}

unconstrained fn f(mut a: [u32; 3], i: u32) -> bool {
    a[i] = 99;
    a[1] == 7
}
