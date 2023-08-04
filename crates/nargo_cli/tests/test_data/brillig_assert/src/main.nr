// Tests a very simple program.
// 
// The features being tested is using assert on brillig
fn main(x: Field)  {
    assert(1 == conditional(x == 1));
}

unconstrained fn conditional(x : bool) -> Field {
    assert(x);
    1
}