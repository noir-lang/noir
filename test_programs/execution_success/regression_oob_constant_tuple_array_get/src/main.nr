// Regression: an out-of-bounds *constant* `array_get` of a field of a tuple, in a Brillig
// function, must keep its result well-typed all the way through Brillig codegen.
//
// `G_A` is an array of 5-field tuples, so it flattens to repeating slots `[str, str, str, Field,
// bool, ...]`. Reading the `bool` field (`.4`) of an out-of-bounds element lowers to a constant
// `array_get` at a flattened slot that is congruent to the `bool` slot. A statically out-of-bounds
// Brillig read is not a hard error: the index is remapped onto an in-bounds slot of the *same*
// field type (the `bool`) behind an "Index out of bounds" trap, so the dead result stays a `bool`.
//
// `brillig_array_get_and_set` then shifts the constant index past the array's in-memory header, and
// folding the read against the backing `make_array` has to map that index back to logical space
// before indexing the elements. Reading the raw element directly lands one slot too far — a `str`
// rather than the `bool` — and the wrongly-typed array value flowing into the `if` condition
// crashes Brillig codegen with "Expected single address".
//
// The index is written as `if false { .. } else { .. }` so it is a runtime value at
// monomorphization (and therefore is not clamped to an in-bounds element there) yet folds to a
// constant in SSA, which is where the out-of-bounds handling and the offset shift both run. The
// access is guarded by the runtime input `run` (false here) so the trap is never reached and the
// program returns `0`. This only reproduces under the minimal SSA pipeline (`--minimal-ssa`),
// where the `make_array` fold happens after the offset shift rather than before it.
global G_A: [(str<4>, str<4>, str<4>, Field, bool); 2] =
    [("FTIU", "XRAD", "BNIZ", 1, true), ("KTDA", "JDVL", "VGCJ", 2, true)];

unconstrained fn main(run: bool) -> pub Field {
    let mut result: Field = 0;
    if run {
        if G_A[if false { 1_u32 } else { 818863061_u32 }].4 {
            result = 1;
        }
    }
    result
}
