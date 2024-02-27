// This program tests:
// - the allocation of virtual arrays for array params to main
// - load instructions for such arrays
fn main(xs: [field; 2]) -> pub field {
    xs[1]
}
