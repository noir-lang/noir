// This program tests:
// - the allocation of virtual arrays for array params to main
// - load instructions for such arrays

fn main(xs : [Field; 2]) -> pub Field {
    xs[1]
}
