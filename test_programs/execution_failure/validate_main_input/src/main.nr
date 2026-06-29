// Entry-point inputs are validated automatically: the compiler injects `vec.validate()` at the
// start of `main` because `BoundedVec` implements `Validate`. Here `vec` is decoded straight from
// the circuit inputs with `len = 5` while its capacity is only `3` (the invariant violation from
// https://github.com/noir-lang/noir-claude/issues/814), so execution fails before the body runs —
// note the body itself does nothing.
fn main(vec: BoundedVec<Field, 3>) {}
