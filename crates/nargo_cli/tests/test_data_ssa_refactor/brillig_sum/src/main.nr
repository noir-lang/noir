// Tests a very simple program.
// 
// The features being tested is basic arithmetics on brillig
fn main(x: Field, y: Field, result: Field)  {
    assert(result == add(x, y));
}

unconstrained fn add(x : Field, y : Field) -> Field {
    x + y
}