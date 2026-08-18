// Regression: storing a nested array with `array_set` must copy the array-valued element
// rather than keep sharing its handle with the source array. Here `outer[idx] = inner` stores
// `inner` into `outer`, then `inner[idx] = val` mutates `inner` in place (a `mut` array set
// after the `Mutable Array Set` pass). If the stored copy aliased `inner`, `outer` would also
// observe the mutation. The correct result is `([[0]], [7])`; the aliasing bug produced
// `([[7]], [7])` in the SSA interpreter.
fn main(idx: u32, val: Field) -> pub ([[Field; 1]; 1], [Field; 1]) {
    let mut inner = [0];
    let mut outer = [[9]];
    outer[idx] = inner;
    inner[idx] = val;
    (outer, inner)
}
