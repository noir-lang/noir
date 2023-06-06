// Tests a very simple program.
// 
// The features being tested is assertion
fn main(x : Field)  {
    assert(x == identity(x));
}

unconstrained fn identity(x : Field) -> Field {
  x
}