// Tests a very simple program.
// 
// The feature being tested is brillig as the entry point.
unconstrained fn main(array: [field; 3], x: pub field) -> pub [field; 2] {
    [array[x], array[x + 1]]
}
