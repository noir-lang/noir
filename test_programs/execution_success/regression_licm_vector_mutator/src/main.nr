// Regression test for loop-invariant code motion hoisting a Brillig vector
// mutator (`vector_push_front`) into the loop pre-header while the `inc_rc`
// guarding its vector operand stays behind in the loop body. Separated from
// its guard, the hoisted push finds the vector at refcount 1 and mutates it
// in place, corrupting `v` for the read after the loop.
// See https://github.com/noir-lang/noir-claude/issues/244.
//
// Each piece below defeats a mechanism that otherwise masks the bug:
// - `v`'s length must be dynamic (the `0..n` push loop), otherwise the vector
//   intrinsics are constant-folded away before LICM runs.
// - `v` must reach the second loop with spare capacity and refcount 1: a push
//   only reuses its operand's storage when the new size fits the capacity.
//   `as_vector()` gives capacity == size, so with `n = 0` the hoisted push
//   harmlessly reallocates; the reallocating `push_back` (with `n = 1`)
//   doubles the capacity, letting the next push mutate in place.
// - `v` must not be used between the two loops: any use there would emit a
//   clone (`inc_rc`) in the pre-header, accidentally guarding the hoisted push.
// - The `0..2` loop is guaranteed to execute (a requirement for hoisting the
//   call) and `acc` keeps the push alive through dead instruction elimination.
unconstrained fn main(x: bool, n: u32) -> pub bool {
    let mut v = [x].as_vector();
    for _ in 0..n {
        v = v.push_back(x);
    }
    let mut acc = 0;
    for _ in 0..2 {
        acc += v.push_front(false).len();
    }
    assert(acc > 0);
    // `push_front` returns a new vector, so `v` itself must be unchanged.
    assert(v[0] == x);
    v[0]
}
