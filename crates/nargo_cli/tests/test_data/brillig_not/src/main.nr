// Tests a very simple Brillig function.
// 
// The features being tested is not instruction on brillig
fn main(x: Field, y : Field)  {
    assert(false == not_operator(x == 1));
    assert(true == not_operator(y == 1));
}

unconstrained fn not_operator(x : bool) -> bool {
    !x
}