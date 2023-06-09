use dep::std;
// Tests a very simple program.
// 
// The features being tested is the identity function in Brillig
fn main(x : Field)  {
    assert(x == identity(x));
    // TODO: add support for array comparison
    let arr = identity_array([x, x]);
    assert(x == arr[0]);
    assert(x == arr[1]);
}

unconstrained fn identity(x : Field) -> Field {
  x
}

unconstrained fn identity_array(arr : [Field; 2]) -> [Field; 2] {
  arr
}