// Tests a very simple program.
// 
// The features being tested is using array equality on brillig
fn main(x: [Field;3], y : [Field;3])  {
    assert(array_eq(x, y));
}

unconstrained fn array_eq(x : [Field;3], y : [Field;3]) -> bool {
    let _z = x[0];
    x == y
}