// Tests a very simple program.
// 
// The features being tested is basic arithmetics on brillig
fn main(x: Field)  {
    assert(4 == conditional(x as bool));
}

unconstrained fn conditional(x : bool) -> Field {
    if x {
        4
    }else {
        5
    }
}