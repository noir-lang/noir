// Tests a very simple program.
// 
// The feature being tested is brillig as the entry point.

unconstrained fn main(array: [Field; 3], x: pub Field) -> pub [Field; 2] {
    [array[x], array[x + 1]]
}
