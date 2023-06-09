use dep::std;
// Tests a very simple program.
// 
// The features being tested is the identity function in Brillig
fn main(x : Field)  {
    assert(x == identity(x));
    identity_array([x, x]);
    // let arr = identity_array([x, x]);
    // let y = arr[0];
    // std::println(y);
}

unconstrained fn identity(x : Field) -> Field {
  x
}

unconstrained fn identity_array(arr : [Field; 2]) -> [Field; 2] {
  arr
}