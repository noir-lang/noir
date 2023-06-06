// Tests a very simple program.
// 
// The features being tested is the identity function in Brillig
fn main(x : Field)  {
    assert(x == identity(x));
}

unconstrained fn identity(x : Field) -> Field {
  x
}