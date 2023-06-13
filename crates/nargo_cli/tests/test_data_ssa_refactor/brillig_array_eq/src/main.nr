// Tests a very simple program.
// 
// The features being tested is using array equality on brillig
fn main(x: [Field;3], y : [Field;3])  {
    assert(y[0] == 1);
    assert(array_eq(x, y));
    assert(!array_eq(x, [1,2,4]));
}

unconstrained fn array_eq(x : [Field;3], y : [Field;3]) -> bool {
    x == y
}