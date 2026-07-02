// The standard library's own `impl Validate for BoundedVec`. `vec` is decoded straight from the
// circuit inputs with `len = 5` while its capacity is only `3` — a length-exceeds-capacity
// violation that the safe `BoundedVec` API can never produce but untrusted input can (see
// https://github.com/noir-lang/noir-claude/issues/814). `validate` rejects it at runtime.
fn main(vec: BoundedVec<Field, 3>) {
    vec.validate();
}
