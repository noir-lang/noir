// Tests a very simple Brillig function.
// 
// The features being tested is not instruction on brillig
fn main(x: field, y: field) {
    assert(false == not_operator(x as bool));
    assert(true == not_operator(y as bool));
}

unconstrained fn not_operator(x: bool) -> bool {
    !x
}
